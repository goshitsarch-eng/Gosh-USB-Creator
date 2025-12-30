use crate::commands::devices::format_size;
use md5::{Digest, Md5};
use serde::Serialize;
use sha2::Sha256;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub size_human: String,
}

#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfo, String> {
    let path_clone = path.clone();

    tokio::task::spawn_blocking(move || {
        let path_ref = Path::new(&path_clone);

        let metadata = std::fs::metadata(&path_clone)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let size = metadata.len();
        let name = path_ref
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_clone.clone());

        Ok(FileInfo {
            path: path_clone,
            name,
            size,
            size_human: format_size(size),
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn calculate_checksum(path: String, algorithm: String) -> Result<String, String> {
    // Run CPU-intensive hashing in a blocking thread to avoid blocking the async runtime
    tokio::task::spawn_blocking(move || {
        let mut file = File::open(&path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        const BUFFER_SIZE: usize = 8 * 1024 * 1024; // 8 MB buffer for better throughput
        let mut buffer = vec![0u8; BUFFER_SIZE];

        match algorithm.to_lowercase().as_str() {
            "sha256" => {
                let mut hasher = Sha256::new();
                loop {
                    let bytes_read = file
                        .read(&mut buffer)
                        .map_err(|e| format!("Failed to read file: {}", e))?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                let result = hasher.finalize();
                Ok(format!("{:x}", result))
            }
            "md5" => {
                let mut hasher = Md5::new();
                loop {
                    let bytes_read = file
                        .read(&mut buffer)
                        .map_err(|e| format!("Failed to read file: {}", e))?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                let result = hasher.finalize();
                Ok(format!("{:x}", result))
            }
            _ => Err(format!("Unsupported algorithm: {}", algorithm)),
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
