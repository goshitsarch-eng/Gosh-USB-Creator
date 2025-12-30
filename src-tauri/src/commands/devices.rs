use crate::platform;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BlockDevice {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub size_human: String,
    pub removable: bool,
    pub mount_points: Vec<String>,
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[tauri::command]
pub async fn list_devices() -> Result<Vec<BlockDevice>, String> {
    let result: Result<Vec<BlockDevice>, crate::platform::PlatformError> =
        platform::list_removable_devices().await;
    result.map_err(|e| e.to_string())
}
