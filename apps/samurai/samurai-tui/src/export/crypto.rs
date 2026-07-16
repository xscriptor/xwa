use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

const HEADER: &[u8] = b"SAMURAI_DB_EXPORT_V1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const PBKDF2_ITERATIONS: u32 = 600_000;
const KEY_LEN: usize = 32;

pub fn encrypt_data(data: &[u8], password: &str) -> Vec<u8> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut key);

    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, data).expect("Encryption failed");

    let mut result = Vec::with_capacity(HEADER.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    result.extend_from_slice(HEADER);
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    result
}

#[allow(dead_code)]
pub fn decrypt_data(encrypted: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let header_len = HEADER.len();
    if encrypted.len() < header_len + SALT_LEN + NONCE_LEN + 1 {
        return Err("Archivo demasiado corto".into());
    }
    if &encrypted[..header_len] != HEADER {
        return Err("Formato de archivo no válido".into());
    }

    let salt = &encrypted[header_len..header_len + SALT_LEN];
    let nonce_bytes = &encrypted[header_len + SALT_LEN..header_len + SALT_LEN + NONCE_LEN];
    let ciphertext = &encrypted[header_len + SALT_LEN + NONCE_LEN..];

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Key error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Contraseña incorrecta o archivo corrupto".into())
}
