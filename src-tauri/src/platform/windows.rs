use super::PlatformError;
use crate::commands::devices::{format_size, BlockDevice};
use std::process::Command;
use tokio::fs::File;

pub async fn list_removable_devices() -> Result<Vec<BlockDevice>, PlatformError> {
    // Use PowerShell to get disk information
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Get-Disk | Where-Object { $_.BusType -eq 'USB' } | ForEach-Object {
                $disk = $_
                $partitions = Get-Partition -DiskNumber $disk.Number -ErrorAction SilentlyContinue
                $mountPoints = @()
                foreach ($p in $partitions) {
                    $vol = Get-Volume -Partition $p -ErrorAction SilentlyContinue
                    if ($vol.DriveLetter) {
                        $mountPoints += "$($vol.DriveLetter):"
                    }
                }
                [PSCustomObject]@{
                    Number = $disk.Number
                    FriendlyName = $disk.FriendlyName
                    Size = $disk.Size
                    MountPoints = $mountPoints -join ","
                }
            } | ConvertTo-Json -Compress"#,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PlatformError::Command(format!(
            "PowerShell command failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }

    // Parse JSON output
    let disks: Vec<DiskInfo> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed)
            .map_err(|e| PlatformError::Parse(format!("Failed to parse disk list: {}", e)))?
    } else {
        // Single disk returns object, not array
        let disk: DiskInfo = serde_json::from_str(trimmed)
            .map_err(|e| PlatformError::Parse(format!("Failed to parse disk info: {}", e)))?;
        vec![disk]
    };

    let devices = disks
        .into_iter()
        .map(|disk| {
            let mount_points: Vec<String> = if disk.mount_points.is_empty() {
                Vec::new()
            } else {
                disk.mount_points
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };

            BlockDevice {
                path: format!(r"\\.\PhysicalDrive{}", disk.number),
                name: disk.friendly_name,
                size: disk.size,
                size_human: format_size(disk.size),
                removable: true,
                mount_points,
            }
        })
        .collect();

    Ok(devices)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DiskInfo {
    number: u32,
    friendly_name: String,
    size: u64,
    mount_points: String,
}

pub async fn unmount_device(device_path: &str) -> Result<(), PlatformError> {
    let disk_num = parse_disk_number(device_path)?;

    // Offline the disk to release all volumes
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                $disk = Get-Disk -Number {}
                $partitions = Get-Partition -DiskNumber {} -ErrorAction SilentlyContinue
                foreach ($p in $partitions) {{
                    $vol = Get-Volume -Partition $p -ErrorAction SilentlyContinue
                    if ($vol.DriveLetter) {{
                        $driveLetter = "$($vol.DriveLetter):"
                        Write-Host "Ejecting $driveLetter"
                        $null = (New-Object -ComObject Shell.Application).Namespace(17).ParseName($driveLetter).InvokeVerb("Eject")
                    }}
                }}
                "#,
                disk_num, disk_num
            ),
        ])
        .output()?;

    // Even if the above fails, we'll try to continue
    // The actual write operation will lock the volume

    Ok(())
}

pub async fn open_device_for_write(device_path: &str) -> Result<File, PlatformError> {
    let _ = parse_disk_number(device_path)?;

    match tokio::fs::OpenOptions::new()
        .write(true)
        .open(device_path)
        .await
    {
        Ok(file) => Ok(file),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(PlatformError::Permission(
                "Permission denied. Run as Administrator to write to disk".to_string(),
            ))
        }
        Err(e) => Err(PlatformError::Io(e)),
    }
}

pub async fn open_device_for_read(device_path: &str) -> Result<File, PlatformError> {
    let _ = parse_disk_number(device_path)?;

    File::open(device_path).await.map_err(PlatformError::Io)
}

pub async fn eject_device(device_path: &str) -> Result<(), PlatformError> {
    let disk_num = parse_disk_number(device_path)?;

    // Eject all volumes on this disk
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                r#"
                $partitions = Get-Partition -DiskNumber {} -ErrorAction SilentlyContinue
                foreach ($p in $partitions) {{
                    $vol = Get-Volume -Partition $p -ErrorAction SilentlyContinue
                    if ($vol.DriveLetter) {{
                        $driveLetter = "$($vol.DriveLetter):"
                        Write-Host "Ejecting $driveLetter"
                        $null = (New-Object -ComObject Shell.Application).Namespace(17).ParseName($driveLetter).InvokeVerb("Eject")
                        Start-Sleep -Milliseconds 500
                    }}
                }}
                "#,
                disk_num
            ),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PlatformError::Command(format!(
            "Failed to eject device: {}",
            stderr
        )));
    }

    Ok(())
}

fn parse_disk_number(device_path: &str) -> Result<u32, PlatformError> {
    let disk_num = device_path
        .strip_prefix(r"\\.\PhysicalDrive")
        .ok_or_else(|| PlatformError::Parse("Invalid device path".to_string()))?;

    disk_num
        .trim()
        .parse::<u32>()
        .map_err(|_| PlatformError::Parse("Invalid device path".to_string()))
}
