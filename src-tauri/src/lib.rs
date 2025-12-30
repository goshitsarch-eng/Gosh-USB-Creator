mod commands;
mod platform;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::devices::list_devices,
            commands::verify::get_file_info,
            commands::verify::calculate_checksum,
            commands::verify::validate_image,
            commands::write::write_iso_to_device,
            commands::write::eject_device,
        ])
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                // DevTools will auto-open in debug mode
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
