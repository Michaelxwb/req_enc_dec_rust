use pyo3::prelude::*;

mod crypto;
mod middleware;

use crypto::{AESCipher, DESCipher};
use middleware::EncryptionPlugin;

#[pymodule]
fn req_enc_dec_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AESCipher>()?;
    m.add_class::<DESCipher>()?;
    m.add_class::<EncryptionPlugin>()?;
    Ok(())
}