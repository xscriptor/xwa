use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;

#[derive(Clone)]
pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    pub async fn save_file(&self, rel_path: &str, data: &[u8]) -> Result<String, String> {
        let full_path = self.base_path.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create dirs: {}", e))?;
        }
        fs::write(&full_path, data)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(rel_path.to_string())
    }

    pub async fn read_file(&self, rel_path: &str) -> Result<Vec<u8>, String> {
        let full_path = self.join_safe(rel_path)?;
        fs::read(&full_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))
    }

    pub async fn list_files(&self, prefix: &str) -> Result<Vec<FileInfo>, String> {
        let dir = self.base_path.join(prefix);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        let mut dirs = vec![dir.clone()];

        while let Some(current) = dirs.pop() {
            let mut read_dir = fs::read_dir(&current)
                .await
                .map_err(|e| format!("Failed to read dir: {}", e))?;
            while let Some(entry) = read_dir.next_entry().await.map_err(|e| format!("{}", e))? {
                let path = entry.path();
                let rel = path.strip_prefix(&dir).unwrap_or(&path);
                let rel_str = rel.to_string_lossy().to_string();
                let metadata = entry.metadata().await.map_err(|e| format!("{}", e))?;
                let modified = metadata.modified()
                    .map(|t| SystemTime::now().duration_since(t).unwrap_or_default().as_secs())
                    .unwrap_or(0);
                let is_dir = metadata.is_dir();
                if is_dir {
                    dirs.push(path);
                }
                result.push(FileInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: rel_str,
                    is_dir,
                    size: metadata.len(),
                    modified,
                });
            }
        }

        Ok(result)
    }

    fn join_safe(&self, rel_path: &str) -> Result<PathBuf, String> {
        let cleaned = rel_path.strip_prefix('/').unwrap_or(rel_path);
        let full = self.base_path.join(cleaned);
        if !full.starts_with(&self.base_path) {
            return Err("Path traversal detected".into());
        }
        Ok(full)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
}
