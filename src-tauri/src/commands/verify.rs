use crate::commands::devices::format_size;
use md5::{Digest, Md5};
use serde::Serialize;
use sha2::Sha256;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub size_human: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageValidation {
    pub is_valid: bool,
    pub format: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
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

#[tauri::command]
pub async fn validate_image(path: String, device_size: Option<u64>) -> Result<ImageValidation, String> {
    tokio::task::spawn_blocking(move || {
        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut format = String::from("Unknown");

        let mut file = File::open(&path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let metadata = file.metadata()
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        let file_size = metadata.len();

        // Check if file size exceeds device capacity
        if let Some(dev_size) = device_size {
            if file_size > dev_size {
                errors.push(format!(
                    "Image size ({}) exceeds device capacity ({})",
                    format_size(file_size),
                    format_size(dev_size)
                ));
            }
        }

        // Check for ISO 9660 format
        // Primary Volume Descriptor is at sector 16 (offset 32768)
        // Bytes 0: type (0x01 for primary)
        // Bytes 1-5: "CD001"
        let mut iso_header = [0u8; 6];
        if file.seek(SeekFrom::Start(32768)).is_ok() {
            if file.read_exact(&mut iso_header).is_ok() {
                if &iso_header[1..6] == b"CD001" {
                    format = String::from("ISO 9660");
                }
            }
        }

        // If not ISO 9660, check for raw disk image signatures
        if format == "Unknown" {
            // Check for MBR boot signature at offset 510-511 (0x55, 0xAA)
            file.seek(SeekFrom::Start(510))
                .map_err(|e| format!("Failed to seek: {}", e))?;
            let mut mbr_sig = [0u8; 2];
            if file.read_exact(&mut mbr_sig).is_ok() {
                if mbr_sig == [0x55, 0xAA] {
                    format = String::from("Disk Image (MBR)");
                }
            }
        }

        // Check for GPT
        if format == "Unknown" || format == "Disk Image (MBR)" {
            // GPT header is at LBA 1 (offset 512)
            // Signature is "EFI PART" at offset 512
            file.seek(SeekFrom::Start(512))
                .map_err(|e| format!("Failed to seek: {}", e))?;
            let mut gpt_sig = [0u8; 8];
            if file.read_exact(&mut gpt_sig).is_ok() {
                if &gpt_sig == b"EFI PART" {
                    format = String::from("Disk Image (GPT)");
                }
            }
        }

        // Warn if format is unknown
        if format == "Unknown" {
            warnings.push(String::from("Could not detect image format. File may not be a valid disk image."));
        }

        // Check minimum file size (at least 512 bytes for a boot sector)
        if file_size < 512 {
            errors.push(String::from("File is too small to be a valid disk image."));
        }

        let is_valid = errors.is_empty();

        Ok(ImageValidation {
            is_valid,
            format,
            errors,
            warnings,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
