use super::PlatformError;
use crate::commands::devices::{format_size, BlockDevice};
use std::fs;
use std::path::Path;
use std::process::Command;
use tokio::fs::File;

pub async fn list_removable_devices() -> Result<Vec<BlockDevice>, PlatformError> {
    let mut devices = Vec::new();

    let block_dir = Path::new("/sys/block");
    if !block_dir.exists() {
        return Ok(devices);
    }

    for entry in fs::read_dir(block_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip non-disk devices (loop, ram, etc.)
        if name.starts_with("loop")
            || name.starts_with("ram")
            || name.starts_with("zram")
            || name.starts_with("dm-")
        {
            continue;
        }

        let device_path = entry.path();

        // Check if removable
        let removable_path = device_path.join("removable");
        let removable = fs::read_to_string(&removable_path)
            .map(|s| s.trim() == "1")
            .unwrap_or(false);

        if !removable {
            continue;
        }

        // Get size
        let size_path = device_path.join("size");
        let size_sectors: u64 = fs::read_to_string(&size_path)
            .map(|s| s.trim().parse().unwrap_or(0))
            .unwrap_or(0);
        let size = size_sectors * 512; // Sector size is typically 512 bytes

        if size == 0 {
            continue;
        }

        // Get device model/name
        let model_path = device_path.join("device/model");
        let vendor_path = device_path.join("device/vendor");
        let model = fs::read_to_string(&model_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let vendor = fs::read_to_string(&vendor_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let device_name = if !vendor.is_empty() && !model.is_empty() {
            format!("{} {}", vendor, model)
        } else if !model.is_empty() {
            model
        } else if !vendor.is_empty() {
            vendor
        } else {
            name.clone()
        };

        // Get mount points
        let mount_points = get_mount_points(&format!("/dev/{}", name));

        devices.push(BlockDevice {
            path: format!("/dev/{}", name),
            name: device_name,
            size,
            size_human: format_size(size),
            removable: true,
            mount_points,
        });
    }

    Ok(devices)
}

fn get_mount_points(device_path: &str) -> Vec<String> {
    let mut mounts = Vec::new();

    if let Ok(content) = fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mounted_device = parts[0];
                let mount_point = parts[1];

                // Check if this mount is for our device or a partition
                if mounted_device.starts_with(device_path) {
                    mounts.push(mount_point.to_string());
                }
            }
        }
    }

    mounts
}

pub async fn unmount_device(device_path: &str) -> Result<(), PlatformError> {
    // Get all mount points for this device
    let mounts = get_mount_points(device_path);

    for mount_point in mounts {
        let output = Command::new("umount").arg(&mount_point).output()?;

        if !output.status.success() {
            // Try with sudo/pkexec if normal unmount fails
            let output = Command::new("pkexec")
                .args(["umount", &mount_point])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(PlatformError::Command(format!(
                    "Failed to unmount {}: {}",
                    mount_point, stderr
                )));
            }
        }
    }

    Ok(())
}

pub async fn open_device_for_write(device_path: &str) -> Result<File, PlatformError> {
    // Validate path
    if !device_path.starts_with("/dev/") {
        return Err(PlatformError::Permission(
            "Invalid device path".to_string(),
        ));
    }

    // Try to open directly first
    match tokio::fs::OpenOptions::new()
        .write(true)
        .open(device_path)
        .await
    {
        Ok(file) => Ok(file),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(PlatformError::Permission(format!(
                "Permission denied. Run with elevated privileges to write to {}",
                device_path
            )))
        }
        Err(e) => Err(PlatformError::Io(e)),
    }
}

pub async fn open_device_for_read(device_path: &str) -> Result<File, PlatformError> {
    if !device_path.starts_with("/dev/") {
        return Err(PlatformError::Permission(
            "Invalid device path".to_string(),
        ));
    }

    File::open(device_path)
        .await
        .map_err(PlatformError::Io)
}
