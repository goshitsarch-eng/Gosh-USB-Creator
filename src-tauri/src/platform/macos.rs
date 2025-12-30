use super::PlatformError;
use crate::commands::devices::{format_size, BlockDevice};
use plist::Value;
use std::process::Command;
use tokio::fs::File;

pub async fn list_removable_devices() -> Result<Vec<BlockDevice>, PlatformError> {
    let output = Command::new("diskutil")
        .args(["list", "-plist", "external", "physical"])
        .output()?;

    if !output.status.success() {
        return Err(PlatformError::Command(
            "Failed to run diskutil".to_string(),
        ));
    }

    let plist: Value = plist::from_bytes(&output.stdout)
        .map_err(|e| PlatformError::Parse(format!("Failed to parse diskutil output: {}", e)))?;

    let mut devices = Vec::new();

    if let Some(all_disks) = plist.as_dictionary().and_then(|d| d.get("AllDisks")) {
        if let Some(disks) = all_disks.as_array() {
            for disk in disks {
                if let Some(disk_id) = disk.as_string() {
                    // Skip partitions (e.g., disk2s1), only process whole disks
                    if disk_id.contains('s') {
                        continue;
                    }

                    if let Ok(device) = get_disk_info(disk_id).await {
                        if device.removable {
                            devices.push(device);
                        }
                    }
                }
            }
        }
    }

    Ok(devices)
}

async fn get_disk_info(disk_id: &str) -> Result<BlockDevice, PlatformError> {
    let output = Command::new("diskutil")
        .args(["info", "-plist", disk_id])
        .output()?;

    if !output.status.success() {
        return Err(PlatformError::Command(format!(
            "Failed to get info for {}",
            disk_id
        )));
    }

    let plist: Value = plist::from_bytes(&output.stdout)
        .map_err(|e| PlatformError::Parse(format!("Failed to parse disk info: {}", e)))?;

    let dict = plist
        .as_dictionary()
        .ok_or_else(|| PlatformError::Parse("Invalid disk info format".to_string()))?;

    let device_path = format!("/dev/{}", disk_id);

    let name = dict
        .get("MediaName")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            dict.get("IORegistryEntryName")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .unwrap_or_else(|| disk_id.to_string())
        });

    let size = dict
        .get("TotalSize")
        .and_then(|v| v.as_unsigned_integer())
        .unwrap_or(0);

    let removable = dict
        .get("Removable")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false)
        || dict
            .get("RemovableMedia")
            .and_then(|v| v.as_boolean())
            .unwrap_or(false);

    let mount_point = dict
        .get("MountPoint")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());

    let mount_points = if let Some(mp) = mount_point {
        if mp.is_empty() {
            Vec::new()
        } else {
            vec![mp]
        }
    } else {
        Vec::new()
    };

    Ok(BlockDevice {
        path: device_path,
        name,
        size,
        size_human: format_size(size),
        removable,
        mount_points,
    })
}

pub async fn unmount_device(device_path: &str) -> Result<(), PlatformError> {
    // Extract disk identifier from path (e.g., /dev/disk2 -> disk2)
    let disk_id = device_path
        .strip_prefix("/dev/")
        .or_else(|| device_path.strip_prefix("/dev/r"))
        .ok_or_else(|| PlatformError::Parse("Invalid device path".to_string()))?;

    let output = Command::new("diskutil")
        .args(["unmountDisk", disk_id])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PlatformError::Command(format!(
            "Failed to unmount disk: {}",
            stderr
        )));
    }

    Ok(())
}

pub async fn open_device_for_write(device_path: &str) -> Result<File, PlatformError> {
    // On macOS, use raw device for faster writes
    let raw_path = if device_path.starts_with("/dev/disk") {
        device_path.replace("/dev/disk", "/dev/rdisk")
    } else if device_path.starts_with("/dev/rdisk") {
        device_path.to_string()
    } else {
        return Err(PlatformError::Permission(
            "Invalid device path".to_string(),
        ));
    };

    match tokio::fs::OpenOptions::new()
        .write(true)
        .open(&raw_path)
        .await
    {
        Ok(file) => Ok(file),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(PlatformError::Permission(format!(
                "Permission denied. Run with sudo to write to {}",
                raw_path
            )))
        }
        Err(e) => Err(PlatformError::Io(e)),
    }
}

pub async fn open_device_for_read(device_path: &str) -> Result<File, PlatformError> {
    let raw_path = if device_path.starts_with("/dev/disk") {
        device_path.replace("/dev/disk", "/dev/rdisk")
    } else if device_path.starts_with("/dev/rdisk") {
        device_path.to_string()
    } else {
        return Err(PlatformError::Permission(
            "Invalid device path".to_string(),
        ));
    };

    File::open(&raw_path).await.map_err(PlatformError::Io)
}
