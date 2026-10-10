use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    process::Command,
};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub identity: String,
    pub mount_points: Vec<String>,
    #[serde(default = "default_sector_size")]
    pub sector_size: u32,
}
fn default_sector_size() -> u32 {
    512
}
pub fn command(program: &str, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        return Err(Error::new(
            "platform",
            format!(
                "{program}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(output.stdout)
}
pub fn confirmed(device: &Device) -> Result<Device> {
    devices()?
        .into_iter()
        .find(|d| d.path == device.path && d.identity == device.identity && d.size == device.size)
        .ok_or_else(|| {
            Error::new(
                "device_changed",
                "The selected device changed or was disconnected. Refresh and select it again.",
            )
        })
}
#[cfg(target_os = "linux")]
mod os {
    use super::*;
    use std::{
        fs,
        os::unix::fs::{FileTypeExt, MetadataExt},
        path::{Path, PathBuf},
    };
    fn read(path: impl AsRef<Path>) -> String {
        fs::read_to_string(path)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
    fn decode_mount(s: &str) -> String {
        s.replace("\\040", " ")
            .replace("\\011", "\t")
            .replace("\\012", "\n")
            .replace("\\134", "\\")
    }
    pub fn mounts(paths: &[String], table: &str) -> Vec<String> {
        table
            .lines()
            .filter_map(|l| {
                let mut s = l.split_whitespace();
                let p = s.next()?;
                let m = s.next()?;
                paths.iter().any(|x| x == p).then(|| decode_mount(m))
            })
            .collect()
    }
    pub fn devices() -> Result<Vec<Device>> {
        let table = fs::read_to_string("/proc/mounts")?;
        let mut devices = Vec::new();
        for entry in fs::read_dir("/sys/block")? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let sys = entry.path();
            let canonical = fs::canonicalize(&sys)?;
            let removable = read(sys.join("removable")) == "1";
            let usb = canonical
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with("usb"));
            if !(removable || usb)
                || name.starts_with("loop")
                || name.starts_with("dm-")
                || read(sys.join("ro")) == "1"
            {
                continue;
            }
            let size = read(sys.join("size"))
                .parse::<u64>()
                .unwrap_or(0)
                .checked_mul(512)
                .unwrap_or(0);
            if size == 0 {
                continue;
            }
            let path = format!("/dev/{name}");
            let meta = match fs::metadata(&path) {
                Ok(m) if m.file_type().is_block_device() => m,
                _ => continue,
            };
            let mut paths = vec![path.clone()];
            let mut stacked = fs::read_dir(sys.join("holders"))?.next().is_some();
            for part in fs::read_dir(&sys)? {
                let part = part?;
                if part.path().join("partition").exists() {
                    stacked |= fs::read_dir(part.path().join("holders"))?.next().is_some();
                    paths.push(format!("/dev/{}", part.file_name().to_string_lossy()));
                }
            }
            if stacked {
                // Active device-mapper/LVM/RAID holders must be dismantled by
                // their owner before this whole physical drive can be a target.
                continue;
            }
            let mount_points = mounts(&paths, &table);
            if mount_points
                .iter()
                .any(|m| m == "/" || m == "/boot" || m == "/boot/efi")
            {
                continue;
            }
            let serial = canonical
                .ancestors()
                .map(|p| read(p.join("serial")))
                .find(|s| !s.is_empty())
                .unwrap_or_default();
            let model = format!(
                "{} {}",
                read(sys.join("device/vendor")),
                read(sys.join("device/model"))
            )
            .trim()
            .to_owned();
            let connection = canonical
                .ancestors()
                .find_map(|p| {
                    let number = read(p.join("devnum"));
                    (!number.is_empty()).then(|| format!("{}:{number}", read(p.join("busnum"))))
                })
                .unwrap_or_else(|| format!("{}:{}", meta.ctime(), meta.ctime_nsec()));
            let sector_size = read(sys.join("queue/logical_block_size"))
                .parse()
                .unwrap_or(512);
            let identity = format!(
                "{}:{}:{}:{}:{}:{}",
                canonical.display(),
                serial,
                meta.rdev(),
                size,
                connection,
                sector_size
            );
            devices.push(Device {
                path,
                name: if model.is_empty() { name } else { model },
                size,
                identity,
                mount_points,
                sector_size,
            });
        }
        devices.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(devices)
    }
    fn dbus_error(e: impl std::fmt::Display) -> Error {
        Error::new("permission", format!("Device authorization failed: {e}. Ensure UDisks2 and a desktop authentication agent are running."))
    }
    fn object(path: &str) -> Result<String> {
        let name = path
            .strip_prefix("/dev/")
            .filter(|n| !n.is_empty() && !n.contains('/'))
            .ok_or_else(|| Error::new("validation", "Invalid device path."))?;
        let escaped: String = name
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() {
                    (b as char).to_string()
                } else {
                    format!("_{b:02x}")
                }
            })
            .collect();
        Ok(format!("/org/freedesktop/UDisks2/block_devices/{escaped}"))
    }
    pub fn unmount(device: &Device) -> Result<()> {
        if device.mount_points.is_empty() {
            return Ok(());
        }
        let table = fs::read_to_string("/proc/mounts")?;
        let connection = zbus::blocking::Connection::system();
        for mount in &device.mount_points {
            let path = table
                .lines()
                .find_map(|line| {
                    let mut parts = line.split_whitespace();
                    let source = parts.next()?;
                    let point = parts.next()?;
                    (decode_mount(point) == *mount).then_some(source)
                })
                .ok_or_else(|| {
                    Error::new(
                        "device_changed",
                        "Device mounts changed. Refresh before writing.",
                    )
                })?;
            if let Ok(ref connection) = connection {
                let obj = object(path)?;
                let proxy = zbus::blocking::Proxy::new(
                    connection,
                    "org.freedesktop.UDisks2",
                    obj.as_str(),
                    "org.freedesktop.UDisks2.Filesystem",
                )
                .map_err(dbus_error)?;
                let options: std::collections::HashMap<&str, zbus::zvariant::Value> =
                    std::collections::HashMap::new();
                proxy
                    .call::<_, _, ()>("Unmount", &(options,))
                    .map_err(dbus_error)?;
            } else {
                command("umount", &["--", mount])?;
            }
        }
        Ok(())
    }
    pub fn open(device: &Device) -> Result<(File, Vec<File>)> {
        use std::os::unix::fs::OpenOptionsExt;
        let direct = OpenOptions::new()
            .write(true)
            .read(true)
            .custom_flags(libc::O_EXCL | libc::O_CLOEXEC | libc::O_SYNC)
            .open(&device.path);
        let file = match direct {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                let connection = zbus::blocking::Connection::system().map_err(dbus_error)?;
                let obj = object(&device.path)?;
                let proxy = zbus::blocking::Proxy::new(
                    &connection,
                    "org.freedesktop.UDisks2",
                    obj.as_str(),
                    "org.freedesktop.UDisks2.Block",
                )
                .map_err(dbus_error)?;
                let mut options: std::collections::HashMap<&str, zbus::zvariant::Value> =
                    std::collections::HashMap::new();
                options.insert(
                    "flags",
                    (libc::O_EXCL | libc::O_CLOEXEC | libc::O_SYNC).into(),
                );
                let fd: zbus::zvariant::OwnedFd = proxy
                    .call("OpenDevice", &("rw", options))
                    .map_err(dbus_error)?;
                let fd: std::os::fd::OwnedFd = fd.into();
                File::from(fd)
            }
            Err(e) => return Err(e.into()),
        };
        if !file.metadata()?.file_type().is_block_device() {
            return Err(Error::new(
                "validation",
                "Target is no longer a block device.",
            ));
        }
        let current = confirmed(device)?;
        if file.metadata()?.rdev() != fs::metadata(PathBuf::from(current.path))?.rdev() {
            return Err(Error::new(
                "device_changed",
                "Device changed while opening.",
            ));
        }
        Ok((file, vec![]))
    }
    pub fn eject(device: &Device) -> Result<()> {
        unmount(device)?;
        let connection = zbus::blocking::Connection::system().map_err(dbus_error)?;
        let obj = object(&device.path)?;
        let block = zbus::blocking::Proxy::new(
            &connection,
            "org.freedesktop.UDisks2",
            obj.as_str(),
            "org.freedesktop.UDisks2.Block",
        )
        .map_err(dbus_error)?;
        let drive: zbus::zvariant::OwnedObjectPath =
            block.get_property("Drive").map_err(dbus_error)?;
        let proxy = zbus::blocking::Proxy::new(
            &connection,
            "org.freedesktop.UDisks2",
            drive.as_str(),
            "org.freedesktop.UDisks2.Drive",
        )
        .map_err(dbus_error)?;
        let options: std::collections::HashMap<&str, zbus::zvariant::Value> =
            std::collections::HashMap::new();
        if proxy
            .get_property::<bool>("CanPowerOff")
            .map_err(dbus_error)?
        {
            proxy
                .call::<_, _, ()>("PowerOff", &(options,))
                .map_err(dbus_error)
        } else {
            proxy
                .call::<_, _, ()>("Eject", &(options,))
                .map_err(dbus_error)
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn exact_mount_match_and_escapes() {
            assert_eq!(
                mounts(
                    &["/dev/sda".into(), "/dev/sda1".into()],
                    "/dev/sdaa1 /wrong ext4 rw 0 0\n/dev/sda1 /media/a\\040b vfat rw 0 0"
                ),
                vec!["/media/a b"]
            );
        }
    }
}
#[cfg(target_os = "macos")]
mod os {
    use super::*;
    use plist::Value;
    fn info(id: &str) -> Result<Value> {
        plist::from_bytes(&command("diskutil", &["info", "-plist", id])?)
            .map_err(|e| Error::new("parse", e.to_string()))
    }
    fn string(v: &Value, k: &str) -> String {
        v.as_dictionary()
            .and_then(|d| d.get(k))
            .and_then(Value::as_string)
            .unwrap_or("")
            .into()
    }
    fn boolean(v: &Value, k: &str) -> bool {
        v.as_dictionary()
            .and_then(|d| d.get(k))
            .and_then(Value::as_boolean)
            .unwrap_or(false)
    }
    fn apfs_mounts(apfs: &Value, id: &str) -> Result<Vec<String>> {
        let mut mounts = vec![];
        let containers = apfs
            .as_dictionary()
            .and_then(|d| d.get("Containers"))
            .and_then(Value::as_array)
            .ok_or_else(|| Error::new("parse", "APFS container information is incomplete."))?;
        {
            for container in containers {
                let stores = container
                    .as_dictionary()
                    .and_then(|d| d.get("PhysicalStores"))
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        Error::new("parse", "APFS physical-store information is incomplete.")
                    })?;
                if stores.iter().any(|store| {
                    let device = string(store, "DeviceIdentifier");
                    device == id || device.starts_with(&format!("{id}s"))
                }) {
                    let volumes = container
                        .as_dictionary()
                        .and_then(|d| d.get("Volumes"))
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            Error::new("parse", "APFS volume information is incomplete.")
                        })?;
                    for volume in volumes {
                        let mount = string(volume, "MountPoint");
                        if !mount.is_empty() {
                            mounts.push(mount);
                        }
                    }
                }
            }
        }
        Ok(mounts)
    }
    pub fn devices() -> Result<Vec<Device>> {
        let list: Value = plist::from_bytes(&command(
            "diskutil",
            &["list", "-plist", "external", "physical"],
        )?)
        .map_err(|e| Error::new("parse", e.to_string()))?;
        let apfs: Value = plist::from_bytes(&command("diskutil", &["apfs", "list", "-plist"])?)
            .map_err(|e| Error::new("parse", e.to_string()))?;
        let mut result = vec![];
        if let Some(disks) = list
            .as_dictionary()
            .and_then(|d| d.get("AllDisks"))
            .and_then(Value::as_array)
        {
            for value in disks {
                if let Some(id) = value.as_string() {
                    if !id
                        .strip_prefix("disk")
                        .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
                    {
                        continue;
                    }
                    let v = info(id)?;
                    if boolean(&v, "Internal") || !boolean(&v, "Whole") || !boolean(&v, "Writable")
                    {
                        continue;
                    }
                    let size = v
                        .as_dictionary()
                        .and_then(|d| d.get("TotalSize"))
                        .and_then(Value::as_unsigned_integer)
                        .unwrap_or(0);
                    if size == 0 {
                        continue;
                    }
                    let name = string(&v, "MediaName");
                    let sector_size = v
                        .as_dictionary()
                        .and_then(|d| d.get("DeviceBlockSize"))
                        .and_then(Value::as_unsigned_integer)
                        .and_then(|n| u32::try_from(n).ok())
                        .unwrap_or(512);
                    let identity = format!(
                        "{id}:{name}:{size}:{}:{sector_size}",
                        string(&v, "DeviceTreePath")
                    );
                    let mut mounts = apfs_mounts(&apfs, id)?;
                    let mount = string(&v, "MountPoint");
                    if !mount.is_empty() {
                        mounts.push(mount);
                    }
                    for partition in disks
                        .iter()
                        .filter_map(Value::as_string)
                        .filter(|p| p.starts_with(&format!("{id}s")))
                    {
                        let partition_info = info(partition)?;
                        let mount = string(&partition_info, "MountPoint");
                        if !mount.is_empty() {
                            mounts.push(mount);
                        }
                    }
                    if mounts
                        .iter()
                        .any(|m| m == "/" || m == "/System/Volumes/Data")
                    {
                        continue;
                    }
                    result.push(Device {
                        path: format!("/dev/{id}"),
                        name,
                        size,
                        identity,
                        mount_points: mounts,
                        sector_size,
                    });
                }
            }
        }
        Ok(result)
    }
    pub fn unmount(d: &Device) -> Result<()> {
        command("diskutil", &["unmountDisk", &d.path]).map(|_| ())
    }
    pub fn open(d: &Device) -> Result<(File, Vec<File>)> {
        Ok((
            OpenOptions::new()
                .write(true)
                .read(true)
                .open(d.path.replacen("/dev/disk", "/dev/rdisk", 1))?,
            vec![],
        ))
    }
    pub fn eject(d: &Device) -> Result<()> {
        command("diskutil", &["eject", &d.path]).map(|_| ())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn apfs_mounts_follow_physical_stores_without_disk_prefix_collisions() {
            let apfs: Value = plist::from_bytes(br#"<plist version="1.0"><dict><key>Containers</key><array><dict><key>PhysicalStores</key><array><dict><key>DeviceIdentifier</key><string>disk1s2</string></dict></array><key>Volumes</key><array><dict><key>MountPoint</key><string>/</string></dict><dict><key>MountPoint</key><string>/System/Volumes/Data</string></dict></array></dict></array></dict></plist>"#).unwrap();
            assert_eq!(
                apfs_mounts(&apfs, "disk1").unwrap(),
                vec!["/", "/System/Volumes/Data"]
            );
            assert!(apfs_mounts(&apfs, "disk10").unwrap().is_empty());
            assert!(apfs_mounts(&Value::Dictionary(Default::default()), "disk1").is_err());
        }
    }
}
#[cfg(target_os = "windows")]
mod os {
    use super::*;
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::{Foundation::HANDLE, System::IO::DeviceIoControl};
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Disk {
        number: u32,
        friendly_name: String,
        size: u64,
        serial_number: String,
        mount_points: Vec<String>,
        #[serde(default = "default_sector_size")]
        logical_sector_size: u32,
    }
    pub fn devices() -> Result<Vec<Device>> {
        let data = command(
            "powershell.exe",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                r#"@(Get-Disk | Where-Object { $_.BusType -eq 'USB' -and -not $_.IsBoot -and -not $_.IsSystem -and -not $_.IsReadOnly -and -not $_.IsOffline } | ForEach-Object { $d=$_; [PSCustomObject]@{Number=$d.Number;FriendlyName=$d.FriendlyName;Size=$d.Size;LogicalSectorSize=$d.LogicalSectorSize;SerialNumber=[string]$d.SerialNumber;MountPoints=@(Get-Partition -DiskNumber $d.Number -ErrorAction Stop | ForEach-Object { $_.AccessPaths } | Where-Object { $_ })} }) | ConvertTo-Json -Compress"#,
            ],
        )?;
        if data.is_empty() {
            return Ok(vec![]);
        }
        let disks: Vec<Disk> = serde_json::from_slice(&data)?;
        Ok(disks
            .into_iter()
            .map(|d| Device {
                path: format!(r"\\.\PhysicalDrive{}", d.number),
                identity: format!(
                    "{}:{}:{}:{}",
                    d.number, d.serial_number, d.size, d.logical_sector_size
                ),
                name: d.friendly_name,
                size: d.size,
                mount_points: d.mount_points,
                sector_size: d.logical_sector_size,
            })
            .collect())
    }
    fn ioctl(file: &File, code: u32) -> Result<()> {
        let mut returned = 0;
        // SAFETY: synchronous call, valid live File handle, no input/output buffers.
        let ok = unsafe {
            DeviceIoControl(
                file.as_raw_handle() as HANDLE,
                code,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
    pub fn unmount(_: &Device) -> Result<()> {
        Ok(())
    } // Locks/dismounts are held by open, not a shell verb.
    pub fn open(d: &Device) -> Result<(File, Vec<File>)> {
        let mut volumes = vec![];
        for mount in &d.mount_points {
            // Every mounted partition has a volume GUID access path. Lock each
            // volume once; folder/drive-letter aliases remain in source checks.
            if !mount.starts_with(r"\\?\Volume{") {
                continue;
            }
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(3)
                .open(mount.trim_end_matches('\\'))?;
            ioctl(&f, 0x00090018)?;
            ioctl(&f, 0x00090020)?;
            volumes.push(f);
        }
        if !d.mount_points.is_empty() && volumes.is_empty() {
            return Err(Error::new(
                "validation",
                "Cannot identify volume handles to lock this drive.",
            ));
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(3)
            .custom_flags(0x80000000)
            .open(&d.path)?;
        Ok((file, volumes))
    }
    pub fn eject(d: &Device) -> Result<()> {
        // Storage eject is a device IOCTL, not an asynchronous Shell.Application verb.
        let (file, _locks) = open(d)?;
        ioctl(&file, 0x002d4808)
    }
}
pub use os::{devices, eject, open, unmount};
