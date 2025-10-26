use openssl::symm::{Cipher, Crypter, Mode};
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
        let cipher = Cipher::aes_256_cbc();
        
        let mut crypter = match Crypter::new(cipher, Mode::Encrypt, &self.key, Some(&self.iv)) {
            Ok(crypter) => crypter,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        let block_size = cipher.block_size();
        let mut ciphertext = vec![0; plaintext.len() + block_size];
        
        let count = match crypter.update(plaintext.as_bytes(), &mut ciphertext) {
            Ok(count) => count,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        let rest = match crypter.finalize(&mut ciphertext[count..]) {
            Ok(rest) => rest,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        ciphertext.truncate(count + rest);
        
        let mut result = self.iv.clone();
        result.extend_from_slice(&ciphertext);
        
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
        
        let iv = &decoded[..16];
        let encrypted_data = &decoded[16..];
        
        let cipher = Cipher::aes_256_cbc();
        
        let mut crypter = match Crypter::new(cipher, Mode::Decrypt, &self.key, Some(iv)) {
            Ok(crypter) => crypter,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        let block_size = cipher.block_size();
        let mut plaintext = vec![0; encrypted_data.len() + block_size];
        
        let count = match crypter.update(encrypted_data, &mut plaintext) {
            Ok(count) => count,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        let rest = match crypter.finalize(&mut plaintext[count..]) {
            Ok(rest) => rest,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        plaintext.truncate(count + rest);
        
        match String::from_utf8(plaintext) {
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
        let cipher = Cipher::des_cbc();
        
        let mut crypter = match Crypter::new(cipher, Mode::Encrypt, &self.key, Some(&self.iv)) {
            Ok(crypter) => crypter,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        let block_size = cipher.block_size();
        let mut ciphertext = vec![0; plaintext.len() + block_size];
        
        let count = match crypter.update(plaintext.as_bytes(), &mut ciphertext) {
            Ok(count) => count,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        let rest = match crypter.finalize(&mut ciphertext[count..]) {
            Ok(rest) => rest,
            Err(_) => return Ok(plaintext.to_string()),
        };
        
        ciphertext.truncate(count + rest);
        
        let mut result = self.iv.clone();
        result.extend_from_slice(&ciphertext);
        
        Ok(general_purpose::STANDARD.encode(&result))
    }

    pub fn decrypt(&self, ciphertext: &str) -> PyResult<String> {
        let decoded = match general_purpose::STANDARD.decode(ciphertext) {
            Ok(decoded) => decoded,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        if decoded.len() < 8 {
            return Ok(ciphertext.to_string());
        }
        
        let iv = &decoded[..8];
        let encrypted_data = &decoded[8..];
        
        let cipher = Cipher::des_cbc();
        
        let mut crypter = match Crypter::new(cipher, Mode::Decrypt, &self.key, Some(iv)) {
            Ok(crypter) => crypter,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        let block_size = cipher.block_size();
        let mut plaintext = vec![0; encrypted_data.len() + block_size];
        
        let count = match crypter.update(encrypted_data, &mut plaintext) {
            Ok(count) => count,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        let rest = match crypter.finalize(&mut plaintext[count..]) {
            Ok(rest) => rest,
            Err(_) => return Ok(ciphertext.to_string()),
        };
        
        plaintext.truncate(count + rest);
        
        match String::from_utf8(plaintext) {
            Ok(text) => Ok(text),
            Err(_) => Ok(ciphertext.to_string()),
        }
    }
}