use crate::error::{Error, Result};
use md5::{Digest, Md5};
use serde::Serialize;
use sha2::Sha256;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};
#[derive(Debug, Serialize)]
pub struct Image {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub format: String,
    pub recognized: bool,
    pub warnings: Vec<String>,
}
pub fn inspect(path: &Path) -> Result<Image> {
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() < 512 {
        return Err(Error::new(
            "validation",
            "Choose a regular image file of at least 512 bytes.",
        ));
    }
    let mut format = "Unknown";
    let mut bytes = [0; 6];
    file.seek(SeekFrom::Start(32768))?;
    if file.read_exact(&mut bytes).is_ok() && bytes[0] == 1 && &bytes[1..] == b"CD001" {
        format = "ISO 9660";
    }
    if format == "Unknown" {
        file.seek(SeekFrom::Start(510))?;
        let mut mbr = [0; 2];
        if file.read_exact(&mut mbr).is_ok() && mbr == [0x55, 0xaa] {
            format = "Disk Image (MBR)";
        }
        file.seek(SeekFrom::Start(512))?;
        let mut gpt = [0; 8];
        if file.read_exact(&mut gpt).is_ok() && &gpt == b"EFI PART" {
            format = "Disk Image (GPT)";
        }
    }
    let recognized = format != "Unknown";
    Ok(Image {
        path: path.to_string_lossy().into(),
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
        size: meta.len(),
        format: format.into(),
        recognized,
        warnings: if recognized {
            vec!["Signature detected; bootability is not guaranteed.".into()]
        } else {
            vec!["Unrecognized image signature. Confirm the image's origin before writing.".into()]
        },
    })
}
pub fn checksum(path: &Path, algorithm: &str) -> Result<String> {
    let mut file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(Error::new("validation", "Choose a regular file."));
    }
    checksum_file(&mut file, algorithm)
}
pub fn checksum_file(file: &mut File, algorithm: &str) -> Result<String> {
    checksum_file_checked(file, algorithm, || Ok(()))
}
pub fn checksum_file_checked(
    file: &mut File,
    algorithm: &str,
    mut check: impl FnMut() -> Result<()>,
) -> Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut buffer = vec![0; 1024 * 1024];
    match algorithm {
        "sha256" => {
            let mut hash = Sha256::new();
            loop {
                check()?;
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            Ok(format!("{:x}", hash.finalize()))
        }
        "md5" => {
            let mut hash = Md5::new();
            loop {
                check()?;
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            Ok(format!("{:x}", hash.finalize()))
        }
        _ => Err(Error::new(
            "validation",
            "Supported algorithms are sha256 and md5.",
        )),
    }
}
