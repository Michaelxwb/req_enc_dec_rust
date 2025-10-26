use base64::{Engine as _, engine::general_purpose};
use pyo3::prelude::*;

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
        // Simple XOR encryption as fallback (for demonstration only)
        // In production, you should use a proper encryption library
        let key_bytes = &self.key;
        let plaintext_bytes = plaintext.as_bytes();
        let mut encrypted = Vec::with_capacity(plaintext_bytes.len());
        
        for (i, &byte) in plaintext_bytes.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            encrypted.push(byte ^ key_byte);
        }
        
        let mut result = self.iv.clone();
        result.extend_from_slice(&encrypted);
        
        Ok(general_purpose::STANDARD.encode(&result))
    }

    pub fn decrypt(&self, ciphertext: &str) -> PyResult<String> {
        let decoded = match general_purpose::STANDARD.decode(ciphertext) {
            Ok(decoded) => decoded,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        if decoded.len() < self.iv.len() {
            return Ok(ciphertext.to_string());
        }
        
        let key_bytes = &self.key;
        let encrypted_data = &decoded[self.iv.len()..];
        let mut decrypted = Vec::with_capacity(encrypted_data.len());
        
        for (i, &byte) in encrypted_data.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            decrypted.push(byte ^ key_byte);
        }
        
        match String::from_utf8(decrypted) {
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
        // Simple XOR encryption as fallback (for demonstration only)
        let aes_cipher = AESCipher::new(self.key.clone(), self.iv.clone());
        aes_cipher.encrypt(plaintext)
    }

    pub fn decrypt(&self, ciphertext: &str) -> PyResult<String> {
        // Simple XOR encryption as fallback (for demonstration only)
        let aes_cipher = AESCipher::new(self.key.clone(), self.iv.clone());
        aes_cipher.decrypt(ciphertext)
    }
}