pub mod crypto;

use crate::db::connection::DbPool;
use crate::db::operations::build_export_payload;
use std::fs;
use std::path::Path;

pub async fn export_raw_to_file(pool: &DbPool, output_path: &Path) -> Result<(), String> {
    let payload = build_export_payload(pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let json = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("JSON error: {}", e))?;

    fs::write(output_path, json.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;

    log::info!("Exported to {}", output_path.display());
    Ok(())
}

pub async fn export_encrypted_to_file(
    pool: &DbPool,
    output_path: &Path,
    password: &str,
) -> Result<(), String> {
    if password.len() < 4 {
        return Err("Password must be at least 4 characters".into());
    }

    let payload = build_export_payload(pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let json = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("JSON error: {}", e))?;

    let encrypted = crypto::encrypt_data(json.as_bytes(), password);
    fs::write(output_path, &encrypted)
        .map_err(|e| format!("Write error: {}", e))?;

    log::info!("Encrypted export to {}", output_path.display());
    Ok(())
}

#[allow(dead_code)]
pub async fn decrypt_file(input_path: &Path, password: &str) -> Result<Vec<u8>, String> {
    let encrypted = fs::read(input_path)
        .map_err(|e| format!("Read error: {}", e))?;
    crypto::decrypt_data(&encrypted, password)
}
