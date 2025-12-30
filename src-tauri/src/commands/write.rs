use crate::commands::devices::{format_size, BlockDevice};
use crate::platform;
use serde::Serialize;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Serialize)]
pub struct WriteProgressEvent {
    pub phase: String,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
    pub eta_seconds: u64,
}

const BLOCK_SIZE: usize = 4 * 1024 * 1024; // 4 MB blocks

#[tauri::command]
pub async fn write_iso_to_device(
    iso_path: String,
    device_path: String,
    verify: bool,
    app: AppHandle,
) -> Result<(), String> {
    let device = require_removable_device(&device_path).await?;

    // Unmount the device first
    platform::unmount_device(&device_path)
        .await
        .map_err(|e| format!("Failed to unmount device: {}", e))?;

    // Get file size
    let metadata = tokio::fs::metadata(&iso_path)
        .await
        .map_err(|e| format!("Failed to read ISO file: {}", e))?;
    if !metadata.is_file() {
        return Err("Selected image is not a file".to_string());
    }
    let total_bytes = metadata.len();
    if total_bytes == 0 {
        return Err("Selected image is empty".to_string());
    }
    if device.size > 0 && total_bytes > device.size {
        return Err(format!(
            "Image size ({}) exceeds device capacity ({})",
            format_size(total_bytes),
            format_size(device.size)
        ));
    }

    // Open source file
    let mut source = File::open(&iso_path)
        .await
        .map_err(|e| format!("Failed to open ISO file: {}", e))?;

    // Open device for writing
    let mut device = platform::open_device_for_write(&device_path)
        .await
        .map_err(|e| format!("Failed to open device: {}", e))?;

    // Write phase
    let mut buffer = vec![0u8; BLOCK_SIZE];
    let mut bytes_written: u64 = 0;
    let start_time = Instant::now();

    loop {
        let bytes_read = source
            .read(&mut buffer)
            .await
            .map_err(|e| format!("Failed to read ISO: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        device
            .write_all(&buffer[..bytes_read])
            .await
            .map_err(|e| format!("Failed to write to device: {}", e))?;

        bytes_written += bytes_read as u64;

        // Calculate progress
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_bps = if elapsed > 0.0 {
            (bytes_written as f64 / elapsed) as u64
        } else {
            0
        };
        let remaining_bytes = total_bytes.saturating_sub(bytes_written);
        let eta_seconds = if speed_bps > 0 {
            remaining_bytes / speed_bps
        } else {
            0
        };

        let progress = WriteProgressEvent {
            phase: "writing".to_string(),
            bytes_written,
            total_bytes,
            speed_bps,
            eta_seconds,
        };

        let _ = app.emit("write-progress", &progress);
    }

    // Sync to ensure all data is written
    device
        .sync_all()
        .await
        .map_err(|e| format!("Failed to sync device: {}", e))?;

    drop(device);

    // Verification phase
    if verify {
        // Reopen source and device for verification
        let mut source = File::open(&iso_path)
            .await
            .map_err(|e| format!("Failed to open ISO for verification: {}", e))?;

        let mut device = platform::open_device_for_read(&device_path)
            .await
            .map_err(|e| format!("Failed to open device for verification: {}", e))?;

        let mut source_buffer = vec![0u8; BLOCK_SIZE];
        let mut device_buffer = vec![0u8; BLOCK_SIZE];
        let mut bytes_verified: u64 = 0;
        let verify_start = Instant::now();

        loop {
            let source_read = source
                .read(&mut source_buffer)
                .await
                .map_err(|e| format!("Failed to read ISO during verification: {}", e))?;

            if source_read == 0 {
                break;
            }

            device
                .read_exact(&mut device_buffer[..source_read])
                .await
                .map_err(|e| format!("Failed to read device during verification: {}", e))?;

            if source_buffer[..source_read] != device_buffer[..source_read] {
                return Err("Verification failed: data mismatch detected".to_string());
            }

            bytes_verified += source_read as u64;

            // Calculate progress
            let elapsed = verify_start.elapsed().as_secs_f64();
            let speed_bps = if elapsed > 0.0 {
                (bytes_verified as f64 / elapsed) as u64
            } else {
                0
            };
            let remaining_bytes = total_bytes.saturating_sub(bytes_verified);
            let eta_seconds = if speed_bps > 0 {
                remaining_bytes / speed_bps
            } else {
                0
            };

            let progress = WriteProgressEvent {
                phase: "verifying".to_string(),
                bytes_written: bytes_verified,
                total_bytes,
                speed_bps,
                eta_seconds,
            };

            let _ = app.emit("write-progress", &progress);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn eject_device(device_path: String) -> Result<(), String> {
    let _ = require_removable_device(&device_path).await?;
    platform::eject_device(&device_path)
        .await
        .map_err(|e| format!("Failed to eject device: {}", e))
}

async fn require_removable_device(device_path: &str) -> Result<BlockDevice, String> {
    let devices = platform::list_removable_devices()
        .await
        .map_err(|e| e.to_string())?;

    devices
        .into_iter()
        .find(|device| device.path == device_path)
        .ok_or_else(|| format!("Device not found or not removable: {}", device_path))
}
