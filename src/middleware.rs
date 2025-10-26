use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use std::collections::HashMap;
use std::sync::Mutex;
use sha2::{Sha256, Digest};
use crate::crypto::{AESCipher, DESCipher};

#[pyclass]
pub struct EncryptionPlugin {
    config: PyObject,
    cipher_cache: Mutex<HashMap<String, PyObject>>,
    custom_ciphers: PyObject,
    cipher_instance: Mutex<Option<PyObject>>,
}

#[pymethods]
impl EncryptionPlugin {
    #[new]
    #[pyo3(signature = (app=None))]
    pub fn new(py: Python, app: Option<PyObject>) -> PyResult<Self> {
        let config = if let Some(app_obj) = &app {
            let app_ref = app_obj.as_ref(py);
            let config_attr = app_ref.getattr("config")?;
            config_attr.getattr("copy")?.call0()?
        } else {
            PyDict::new(py).into()
        };

        let custom_ciphers: PyObject = PyDict::new(py).into();
        
        let plugin = Self {
            config: config.into(),
            cipher_cache: Mutex::new(HashMap::new()),
            custom_ciphers: custom_ciphers.into(),
            cipher_instance: Mutex::new(None),
        };

        if let Some(app_obj) = app {
            plugin.init_app(py, app_obj)?;
        }

        Ok(plugin)
    }

    pub fn init_app(&self, py: Python, app: PyObject) -> PyResult<()> {
        self.register_middleware(py, app)?;
        Ok(())
    }

    fn register_middleware(&self, py: Python, app: PyObject) -> PyResult<()> {
        let app_ref = app.as_ref(py);
        
        // before_request hook
        let plugin_config = self.config.clone();
        let plugin_custom_ciphers = self.custom_ciphers.clone();
        
        let before_closure = pyo3::types::PyCFunction::new_closure(
            py,
            None,
            None,
            move |_args, _kwargs| {
                Python::with_gil(|py| {
                    let request = py.import("flask")?.getattr("request")?;
                    let path = request.getattr("path")?.extract::<String>()?;
                    
                    let config_dict = plugin_config.as_ref(py);
                    if let Ok(url_configs) = config_dict.get_item("ENCRYPTION_URL_CONFIGS") {
                        if let Ok(url_configs_dict) = url_configs.downcast::<PyDict>() {
                            if let Ok(Some(url_config)) = url_configs_dict.get_item(&path) {
                                if let Ok(url_config_dict) = url_config.downcast::<PyDict>() {
                                    if let Ok(Some(decrypt_fields)) = url_config_dict.get_item("decrypt_fields") {
                                        if let Ok(decrypt_fields_list) = decrypt_fields.downcast::<PyList>() {
                                            if !decrypt_fields_list.is_empty() {
                                                let json_data = request.getattr("get_json")?.call0()?;
                                                
                                                // Create a temporary plugin instance for processing
                                                let temp_plugin = EncryptionPlugin {
                                                    config: plugin_config.clone(),
                                                    cipher_cache: Mutex::new(HashMap::new()),
                                                    custom_ciphers: plugin_custom_ciphers.clone(),
                                                    cipher_instance: Mutex::new(None),
                                                };
                                                
                                                let _ = temp_plugin.process_nested(py, json_data.into(), decrypt_fields.into(), "decrypt")?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok::<PyObject, PyErr>(py.None())
                })
            }
        )?;
        
        app_ref.getattr("before_request")?.call1((before_closure,))?;

        // after_request hook
        let plugin_config = self.config.clone();
        let plugin_custom_ciphers = self.custom_ciphers.clone();
        
        let after_closure = pyo3::types::PyCFunction::new_closure(
            py,
            None,
            None,
            move |args, _kwargs| {
                Python::with_gil(|py| {
                    let response = args.get_item(0)?;
                    let request = py.import("flask")?.getattr("request")?;
                    let path = request.getattr("path")?.extract::<String>()?;
                    
                    let config_dict = plugin_config.as_ref(py);
                    if let Ok(url_configs) = config_dict.get_item("ENCRYPTION_URL_CONFIGS") {
                        if let Ok(url_configs_dict) = url_configs.downcast::<PyDict>() {
                            if let Ok(Some(url_config)) = url_configs_dict.get_item(&path) {
                                if let Ok(url_config_dict) = url_config.downcast::<PyDict>() {
                                    if let Ok(Some(encrypt_fields)) = url_config_dict.get_item("encrypt_fields") {
                                        if let Ok(encrypt_fields_list) = encrypt_fields.downcast::<PyList>() {
                                            if !encrypt_fields_list.is_empty() {
                                                let json_data = response.getattr("get_json")?.call0()?;
                                                
                                                // Create a temporary plugin instance for processing
                                                let temp_plugin = EncryptionPlugin {
                                                    config: plugin_config.clone(),
                                                    cipher_cache: Mutex::new(HashMap::new()),
                                                    custom_ciphers: plugin_custom_ciphers.clone(),
                                                    cipher_instance: Mutex::new(None),
                                                };
                                                
                                                let encrypted_data = temp_plugin.process_nested(py, json_data.into(), encrypt_fields.into(), "encrypt")?;
                                                
                                                let jsonify = py.import("flask")?.getattr("jsonify")?;
                                                let new_response = jsonify.call1((encrypted_data,))?;
                                                response.call_method1("set_data", (new_response.getattr("data")?,))?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok::<PyObject, PyErr>(response.to_object(py))
                })
            }
        )?;
        
        app_ref.getattr("after_request")?.call1((after_closure,))?;
        
        Ok(())
    }

    pub fn process_nested(&self, py: Python, data: PyObject, fields: PyObject, action: &str) -> PyResult<PyObject> {
        let fields_list = fields.as_ref(py);
        if let Ok(fields_iter) = fields_list.iter() {
            for field_item in fields_iter {
                let field_str = field_item?.extract::<String>()?;
                let keys: Vec<String> = field_str.split('.').map(|s| s.to_string()).collect();
                self._recursive_process(py, data.as_ref(py), keys, action)?;
            }
        }
        Ok(data)
    }

    fn _recursive_process(&self, py: Python, data: &PyAny, keys: Vec<String>, action: &str) -> PyResult<()> {
        if keys.is_empty() {
            return Ok(());
        }

        let key = &keys[0];
        let remaining_keys = &keys[1..];

        if let Ok(dict) = data.downcast::<PyDict>() {
            if let Ok(Some(value)) = dict.get_item(&key) {
                if remaining_keys.is_empty() {
                    if let Ok(list) = value.downcast::<PyList>() {
                            let processed_list = PyList::new(py, Vec::<PyObject>::new());
                            for item in list.iter() {
                                let item_str = item.str()?.to_string();
                                let processed = if action == "encrypt" {
                                    self.encrypt(py, &item_str)?
                                } else {
                                    self.decrypt(py, &item_str)?
                                };
                                processed_list.append(processed)?;
                            }
                            dict.set_item(key, processed_list)?;
                    } else {
                        let original_value = value.str()?.to_string();
                        let processed = if action == "encrypt" {
                            self.encrypt(py, &original_value)?
                        } else {
                            self.decrypt(py, &original_value)?
                        };
                        dict.set_item(key, processed)?;
                    }
                } else {
                    self._recursive_process(py, value, remaining_keys.to_vec(), action)?;
                }
            }
        } else if let Ok(list) = data.downcast::<PyList>() {
            for item in list.iter() {
                self._recursive_process(py, item, keys.clone(), action)?;
            }
        }

        Ok(())
    }

    pub fn encrypt(&self, py: Python, plaintext: &str) -> PyResult<String> {
        let cipher = match self.get_cipher_instance(py) {
            Ok(cipher) => cipher,
            Err(_) => return Ok(plaintext.to_string()), // 如果获取加密实例失败，返回原始文本
        };
        
        let cipher_ref = cipher.as_ref(py);
        match cipher_ref.call_method1("encrypt", (plaintext,))?.extract() {
            Ok(encrypted) => Ok(encrypted),
            Err(_) => Ok(plaintext.to_string()), // 如果加密失败，返回原始文本
        }
    }

    pub fn decrypt(&self, py: Python, ciphertext: &str) -> PyResult<String> {
        let cipher = match self.get_cipher_instance(py) {
            Ok(cipher) => cipher,
            Err(_) => return Ok(ciphertext.to_string()), // 如果获取解密实例失败，返回原始文本
        };
        
        let cipher_ref = cipher.as_ref(py);
        match cipher_ref.call_method1("decrypt", (ciphertext,))?.extract() {
            Ok(decrypted) => Ok(decrypted),
            Err(_) => Ok(ciphertext.to_string()), // 如果解密失败，返回原始文本
        }
    }

    fn get_cipher_instance(&self, py: Python) -> PyResult<PyObject> {
        {
            let cache = self.cipher_instance.lock().unwrap();
            if let Some(ref instance) = *cache {
                return Ok(instance.clone());
            }
        }

        let new_instance = self.create_cipher_instance(py)?;
        let mut cache = self.cipher_instance.lock().unwrap();
        *cache = Some(new_instance.clone());
        Ok(new_instance)
    }

    fn create_cipher_instance(&self, py: Python) -> PyResult<PyObject> {
        let config_dict = self.config.as_ref(py);
        let algo = config_dict.get_item("ENCRYPTION_ALGO")
            .unwrap_or_else(|_| PyString::new(py, "AES").into())
            .extract::<String>()?;
        
        let key_bytes = config_dict.get_item("ENCRYPTION_KEY")?
            .extract::<Vec<u8>>()?;
        
        let mut hasher = Sha256::new();
        hasher.update(&key_bytes);
        let key_hash = hasher.finalize();
        let key = key_hash.to_vec();

        let cache_key = format!("{}_", algo) + &hex::encode(&key);
        
        {
            let cache = self.cipher_cache.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let custom_ciphers = self.custom_ciphers.as_ref(py);
        let cipher: PyObject = if let Ok(custom_cipher) = custom_ciphers.get_item(&algo) {
            custom_cipher.call1((key,))?.into()
        } else if algo == "AES" {
            let salt = config_dict.get_item("ENCRYPTION_SALT")?
                .extract::<Vec<u8>>()?;
            
            let mut iv = vec![0u8; 16];
            let salt_len = salt.len().min(16);
            iv[..salt_len].copy_from_slice(&salt[..salt_len]);
            
            let key_trimmed = if key.len() > 32 { key[..32].to_vec() } else { key };
            Py::new(py, AESCipher::new(key_trimmed, iv))?.into_py(py)
        } else if algo == "DES" {
            let salt = config_dict.get_item("ENCRYPTION_SALT")?
                .extract::<Vec<u8>>()?;
            
            let mut iv = vec![0u8; 8];
            let salt_len = salt.len().min(8);
            iv[..salt_len].copy_from_slice(&salt[..salt_len]);
            
            let key_trimmed = if key.len() > 8 { key[..8].to_vec() } else { key };
            Py::new(py, DESCipher::new(key_trimmed, iv))?.into_py(py)
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Unsupported encryption algorithm: {}", algo)
            ));
        };

        let mut cache = self.cipher_cache.lock().unwrap();
        cache.insert(cache_key, cipher.clone());
        Ok(cipher)
    }

    pub fn register_cipher(&self, py: Python, algo_name: &str, cipher_class: PyObject) -> PyResult<()> {
        let custom_ciphers = self.custom_ciphers.as_ref(py);
        custom_ciphers.set_item(algo_name, cipher_class)?;
        Ok(())
    }
}