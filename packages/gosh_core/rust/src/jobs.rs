use crate::{
    error::{Error, Result},
    platform::{self, Device},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::Instant,
};
#[derive(Clone, Debug, Serialize)]
pub struct Progress {
    pub id: u64,
    pub phase: String,
    pub bytes: u64,
    pub total: u64,
    pub speed_bps: u64,
    pub eta_seconds: u64,
    pub error: Option<Error>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct WriteRequest {
    pub source: String,
    pub device: Device,
    pub verify: bool,
    pub auto_eject: bool,
    #[serde(default)]
    pub expected_checksum: Option<String>,
    #[serde(default)]
    pub algorithm: Option<String>,
}
struct Job {
    cancel: AtomicBool,
    state: Mutex<Progress>,
}
static JOBS: OnceLock<Mutex<HashMap<u64, Arc<Job>>>> = OnceLock::new();
static NEXT: AtomicU64 = AtomicU64::new(1);
static BUSY: AtomicBool = AtomicBool::new(false);
fn registry() -> &'static Mutex<HashMap<u64, Arc<Job>>> {
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}
fn job(id: u64) -> Result<Arc<Job>> {
    registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| Error::new("validation", "Unknown operation."))
}
pub fn poll(id: u64) -> Result<Progress> {
    Ok(job(id)?
        .state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone())
}
pub fn cancel(id: u64) -> Result<()> {
    job(id)?.cancel.store(true, Ordering::SeqCst);
    Ok(())
}
pub fn busy() -> bool {
    BUSY.load(Ordering::SeqCst)
}
pub fn start(request: WriteRequest) -> Result<u64> {
    if BUSY
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(Error::new("busy", "A write operation is already running."));
    }
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let job = Arc::new(Job {
        cancel: AtomicBool::new(false),
        state: Mutex::new(Progress {
            id,
            phase: "preparing".into(),
            bytes: 0,
            total: 0,
            speed_bps: 0,
            eta_seconds: 0,
            error: None,
            warnings: vec![],
        }),
    });
    let mut jobs = registry().lock().unwrap_or_else(|e| e.into_inner());
    jobs.retain(|_, j| {
        !matches!(
            j.state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .phase
                .as_str(),
            "complete" | "error" | "cancelled"
        )
    });
    jobs.insert(id, job.clone());
    drop(jobs);
    let spawn = std::thread::Builder::new()
        .name(format!("gosh-write-{id}"))
        .spawn(move || {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| write(&request, &job)))
                    .unwrap_or_else(|_| {
                        Err(Error::new(
                            "internal",
                            "The write worker stopped unexpectedly; the target may be incomplete.",
                        ))
                    });
            let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
            match result {
                Ok(warnings) => {
                    state.phase = "complete".into();
                    state.warnings = warnings;
                }
                Err(error) => {
                    state.phase = if error.kind == "cancelled" {
                        "cancelled"
                    } else {
                        "error"
                    }
                    .into();
                    state.error = Some(error);
                }
            }
            BUSY.store(false, Ordering::SeqCst);
        });
    if let Err(e) = spawn {
        BUSY.store(false, Ordering::SeqCst);
        registry()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        return Err(e.into());
    }
    Ok(id)
}
fn cancelled(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::SeqCst) {
        Err(Error::new(
            "cancelled",
            "Operation cancelled. The target may contain an incomplete image.",
        ))
    } else {
        Ok(())
    }
}
fn source_on_mount(path: &Path, mount: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        fn normalized(value: &str) -> String {
            value
                .replace('/', "\\")
                .trim_start_matches(r"\\?\")
                .trim_end_matches('\\')
                .to_lowercase()
        }
        let source = normalized(&path.to_string_lossy());
        let mount = normalized(mount);
        return source == mount || source.starts_with(&format!("{mount}\\"));
    }
    #[cfg(not(target_os = "windows"))]
    path.starts_with(mount)
}
#[cfg(all(test, target_os = "windows"))]
mod mount_tests {
    use super::*;
    #[test]
    fn folder_mounts_and_verbatim_paths_respect_component_boundaries() {
        assert!(source_on_mount(
            Path::new(r"\\?\C:\Mounted USB\image.img"),
            r"c:\mounted usb\"
        ));
        assert!(!source_on_mount(
            Path::new(r"C:\Mounted USB backup\image.img"),
            r"C:\Mounted USB\"
        ));
        assert!(source_on_mount(Path::new(r"\\?\E:\image.img"), r"e:\"));
    }
}
fn write(request: &WriteRequest, job: &Job) -> Result<Vec<String>> {
    let path = Path::new(&request.source).canonicalize()?;
    let mut source = File::open(&path)?;
    let before = source.metadata()?;
    if !before.is_file() || before.len() < 512 {
        return Err(Error::new(
            "validation",
            "Choose a regular image file of at least 512 bytes.",
        ));
    }
    let device = platform::confirmed(&request.device)?;
    let alignment = device.sector_size as usize;
    if !(512..=65536).contains(&alignment) || !alignment.is_power_of_two() {
        return Err(Error::new(
            "validation",
            "Unsupported logical sector size for this device.",
        ));
    }
    let padded = before
        .len()
        .checked_add(alignment as u64 - 1)
        .map(|n| n / alignment as u64 * alignment as u64);
    if device.size == 0 || padded.is_none_or(|n| n > device.size) {
        return Err(Error::new(
            "capacity",
            "Image exceeds device capacity or capacity is unknown.",
        ));
    }
    for mount in &device.mount_points {
        if source_on_mount(&path, mount) {
            return Err(Error::new(
                "validation",
                "The source image is on the selected target. Choose a source on another disk.",
            ));
        }
    }
    if let Some(expected) = &request.expected_checksum {
        let algorithm = request.algorithm.as_deref().unwrap_or("sha256");
        let actual =
            crate::image::checksum_file_checked(&mut source, algorithm, || cancelled(&job.cancel))?;
        if !actual.eq_ignore_ascii_case(expected.trim()) {
            return Err(Error::new(
                "checksum",
                "Source checksum changed or does not match. No device data was written.",
            ));
        }
    }
    cancelled(&job.cancel)?;
    platform::unmount(&device)?;
    platform::confirmed(&device)?;
    let (mut target, locks) = platform::open(&device)?;
    let mut notify = |phase: &str, bytes: u64, total: u64, start: Instant| {
        let speed = (bytes as f64 / start.elapsed().as_secs_f64().max(0.001)) as u64;
        let mut s = job.state.lock().unwrap_or_else(|e| e.into_inner());
        s.phase = phase.into();
        s.bytes = bytes;
        s.total = total;
        s.speed_bps = speed;
        s.eta_seconds = total.saturating_sub(bytes) / speed.max(1);
    };
    let result = copy_verify(
        &mut source,
        &mut target,
        before.len(),
        request.verify,
        alignment,
        &job.cancel,
        &mut notify,
    );
    if result.is_err() {
        let _ = target.sync_all();
    }
    result?;
    if source.metadata()?.len() != before.len()
        || source.metadata()?.modified()? != before.modified()?
    {
        return Err(Error::new(
            "source_changed",
            "Source changed during writing. Discard the target and retry with a stable image.",
        ));
    }
    drop(target);
    drop(locks);
    let mut warnings = vec![];
    if request.auto_eject {
        if let Err(e) = platform::confirmed(&device).and_then(|d| platform::eject(&d)) {
            warnings.push(format!("Image written, but eject failed: {e}. Eject through the operating system before removal."));
        }
    } else {
        warnings.push(
            "Image written. Eject through the operating system before removing the drive.".into(),
        );
    }
    Ok(warnings)
}
// The generic stream engine is independent of raw-device discovery and FFI.
pub fn copy_verify(
    source: &mut File,
    target: &mut File,
    total: u64,
    verify: bool,
    alignment: usize,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(&str, u64, u64, Instant),
) -> Result<()> {
    if !alignment.is_power_of_two() || alignment > 65536 {
        return Err(Error::new("validation", "Invalid stream alignment."));
    }
    let mut buffer = vec![0; 4 * 1024 * 1024];
    let start = Instant::now();
    let mut bytes = 0;
    source.seek(SeekFrom::Start(0))?;
    target.seek(SeekFrom::Start(0))?;
    while bytes < total {
        cancelled(cancel)?;
        let count = ((total - bytes) as usize).min(buffer.len());
        source.read_exact(&mut buffer[..count]).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::new("source_changed", "Source became shorter while writing.")
            } else {
                e.into()
            }
        })?;
        let aligned = count.next_multiple_of(alignment);
        buffer[count..aligned].fill(0);
        target.write_all(&buffer[..aligned])?;
        bytes += count as u64;
        progress("writing", bytes, total, start);
    }
    if source.read(&mut [0; 1])? != 0 {
        return Err(Error::new("source_changed", "Source grew while writing."));
    }
    target.sync_all()?;
    cancelled(cancel)?;
    if verify {
        source.seek(SeekFrom::Start(0))?;
        target.seek(SeekFrom::Start(0))?;
        let mut readback = vec![0; buffer.len()];
        let start = Instant::now();
        let mut bytes = 0;
        while bytes < total {
            cancelled(cancel)?;
            let count = ((total - bytes) as usize).min(buffer.len());
            source.read_exact(&mut buffer[..count])?;
            let aligned = count.next_multiple_of(alignment);
            target.read_exact(&mut readback[..aligned])?;
            if buffer[..count] != readback[..count]
                || readback[count..aligned].iter().any(|b| *b != 0)
            {
                return Err(Error::new(
                    "verification",
                    "Written bytes do not match the source.",
                ));
            }
            bytes += count as u64;
            progress("verifying", bytes, total, start);
        }
    }
    Ok(())
}
