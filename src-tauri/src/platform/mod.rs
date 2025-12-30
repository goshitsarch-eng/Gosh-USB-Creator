#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::commands::devices::BlockDevice;
use thiserror::Error;
use tokio::fs::File;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed: {0}")]
    Command(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Permission denied: {0}")]
    Permission(String),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}

pub async fn list_removable_devices() -> Result<Vec<BlockDevice>, PlatformError> {
    #[cfg(target_os = "linux")]
    return linux::list_removable_devices().await;

    #[cfg(target_os = "macos")]
    return macos::list_removable_devices().await;

    #[cfg(target_os = "windows")]
    return windows::list_removable_devices().await;

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(PlatformError::Command("Unsupported platform".to_string()))
}

pub async fn unmount_device(device_path: &str) -> Result<(), PlatformError> {
    #[cfg(target_os = "linux")]
    return linux::unmount_device(device_path).await;

    #[cfg(target_os = "macos")]
    return macos::unmount_device(device_path).await;

    #[cfg(target_os = "windows")]
    return windows::unmount_device(device_path).await;

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(PlatformError::Command("Unsupported platform".to_string()))
}

pub async fn open_device_for_write(device_path: &str) -> Result<File, PlatformError> {
    #[cfg(target_os = "linux")]
    return linux::open_device_for_write(device_path).await;

    #[cfg(target_os = "macos")]
    return macos::open_device_for_write(device_path).await;

    #[cfg(target_os = "windows")]
    return windows::open_device_for_write(device_path).await;

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(PlatformError::Command("Unsupported platform".to_string()))
}

pub async fn open_device_for_read(device_path: &str) -> Result<File, PlatformError> {
    #[cfg(target_os = "linux")]
    return linux::open_device_for_read(device_path).await;

    #[cfg(target_os = "macos")]
    return macos::open_device_for_read(device_path).await;

    #[cfg(target_os = "windows")]
    return windows::open_device_for_read(device_path).await;

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err(PlatformError::Command("Unsupported platform".to_string()))
}
