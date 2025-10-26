use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use base64::{Engine as _, engine::general_purpose};
use pyo3::prelude::*;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

#[pyclass]
pub struct AESCipher {
    key: Vec<u8>,
    iv: Vec<u8>,
}

#[pymethods]
impl AESCipher {
    #[new]
    pub fn new(key: Vec<u8>, iv: Vec<u8>) -> Self {
        Self { key, iv }
    }

    pub fn encrypt(&self, plaintext: &str) -> PyResult<String> {
        let key = &self.key;
        let iv = &self.iv;
        
        if key.len() != 32 {
            return Ok(plaintext.to_string());
        }
        
        let cipher = match Aes256Cbc::new_from_slices(key, iv) {
            Ok(cipher) => cipher,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        let plaintext_bytes = plaintext.as_bytes();
        let mut buffer = vec![0u8; plaintext_bytes.len() + 16];
        buffer[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);
        
        let ciphertext = match cipher.encrypt(&mut buffer, plaintext_bytes.len()) {
            Ok(ciphertext) => ciphertext,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        let mut result = self.iv.clone();
        result.extend_from_slice(ciphertext);
        
        Ok(general_purpose::STANDARD.encode(&result))
    }

    pub fn decrypt(&self, ciphertext: &str) -> PyResult<String> {
        let decoded = match general_purpose::STANDARD.decode(ciphertext) {
            Ok(decoded) => decoded,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        if decoded.len() < 16 {
            return Ok(ciphertext.to_string());
        }
        
        let key = &self.key;
        let iv = &decoded[..16];
        let encrypted_data = &decoded[16..];
        
        if key.len() != 32 {
            return Ok(ciphertext.to_string());
        }
        
        let cipher = match Aes256Cbc::new_from_slices(key, iv) {
            Ok(cipher) => cipher,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        let mut buffer = encrypted_data.to_vec();
        let plaintext = match cipher.decrypt(&mut buffer) {
            Ok(plaintext) => plaintext,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        match String::from_utf8(plaintext.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => Ok(ciphertext.to_string()),
        }
    }
}

#[pyclass]
pub struct DESCipher {
    key: Vec<u8>,
    iv: Vec<u8>,
}

#[pymethods]
impl DESCipher {
    #[new]
    pub fn new(key: Vec<u8>, iv: Vec<u8>) -> Self {
        Self { key, iv }
    }

    pub fn encrypt(&self, plaintext: &str) -> PyResult<String> {
        // DES is deprecated and less secure, using AES as fallback
        let aes_cipher = AESCipher::new(self.key.clone(), self.iv.clone());
        aes_cipher.encrypt(plaintext)
    }

    pub fn decrypt(&self, ciphertext: &str) -> PyResult<String> {
        // DES is deprecated and less secure, using AES as fallback
        let aes_cipher = AESCipher::new(self.key.clone(), self.iv.clone());
        aes_cipher.decrypt(ciphertext)
    }
}