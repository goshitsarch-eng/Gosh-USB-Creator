use super::*;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    sync::atomic::AtomicBool,
};
fn fixture(bytes: &[u8]) -> (std::path::PathBuf, File) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let p = std::env::temp_dir().join(format!(
        "gosh test ü {}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&p)
        .unwrap();
    f.write_all(bytes).unwrap();
    (p, f)
}
#[test]
fn signatures_and_unknown_are_not_validity_claims() {
    for (offset, signature, expected) in [
        (510, b"\x55\xaa".as_slice(), "Disk Image (MBR)"),
        (512, b"EFI PART".as_slice(), "Disk Image (GPT)"),
        (32768, b"\x01CD001".as_slice(), "ISO 9660"),
    ] {
        let mut data = vec![0; 40000];
        data[offset..offset + signature.len()].copy_from_slice(signature);
        let (p, _) = fixture(&data);
        let image = image::inspect(&p).unwrap();
        assert_eq!(image.format, expected);
        assert!(image.recognized);
        fs::remove_file(p).unwrap();
    }
    let (p, _) = fixture(&[0; 512]);
    let image = image::inspect(&p).unwrap();
    assert!(!image.recognized);
    assert!(!image.warnings.is_empty());
    fs::remove_file(p).unwrap();
}
#[test]
fn known_checksums_unicode_and_invalid_algorithm() {
    let (p, _) = fixture(b"abc");
    assert_eq!(
        image::checksum(&p, "sha256").unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        image::checksum(&p, "md5").unwrap(),
        "900150983cd24fb0d6963f7d28e17f72"
    );
    assert!(image::checksum(&p, "sha1").is_err());
    fs::remove_file(p).unwrap();
}
#[test]
fn rejects_short_images_and_directories() {
    let (p, _) = fixture(b"");
    assert!(image::inspect(&p).is_err());
    assert!(image::inspect(&std::env::temp_dir()).is_err());
    fs::remove_file(p).unwrap();
}
#[test]
fn preflight_checksum_stops_cooperatively_before_reading_the_whole_source() {
    use std::io::Seek;
    let (path, mut source) = fixture(&vec![23; 3 * 1024 * 1024]);
    let mut checks = 0;
    let error = image::checksum_file_checked(&mut source, "sha256", || {
        checks += 1;
        if checks == 2 {
            Err(error::Error::new("cancelled", "Cancelled"))
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    assert_eq!(error.kind, "cancelled");
    assert!(source.stream_position().unwrap() < source.metadata().unwrap().len());
    drop(source);
    fs::remove_file(path).unwrap();
}
#[test]
fn real_stream_copy_and_verify_multiple_blocks() {
    let data = vec![197; 4 * 1024 * 1024 + 123];
    let (sp, mut source) = fixture(&data);
    let (tp, mut target) = fixture(&[]);
    let mut phases = vec![];
    jobs::copy_verify(
        &mut source,
        &mut target,
        data.len() as u64,
        true,
        1,
        &AtomicBool::new(false),
        &mut |phase, bytes, total, _| {
            assert!(bytes <= total);
            phases.push(phase.to_string());
        },
    )
    .unwrap();
    assert_eq!(fs::read(&tp).unwrap(), data);
    assert!(phases.contains(&"verifying".to_owned()));
    drop(source);
    drop(target);
    fs::remove_file(sp).unwrap();
    fs::remove_file(tp).unwrap();
}
#[test]
fn cancellation_and_source_growth_fail() {
    let (sp, mut source) = fixture(&[1; 512]);
    let (tp, mut target) = fixture(&[]);
    assert_eq!(
        jobs::copy_verify(
            &mut source,
            &mut target,
            512,
            true,
            1,
            &AtomicBool::new(true),
            &mut |_, _, _, _| {}
        )
        .unwrap_err()
        .kind,
        "cancelled"
    );
    assert_eq!(
        jobs::copy_verify(
            &mut source,
            &mut target,
            511,
            false,
            1,
            &AtomicBool::new(false),
            &mut |_, _, _, _| {}
        )
        .unwrap_err()
        .kind,
        "source_changed"
    );
    drop(source);
    drop(target);
    fs::remove_file(sp).unwrap();
    fs::remove_file(tp).unwrap();
}
#[test]
fn ffi_malformed_null_and_unknown_requests_are_errors() {
    unsafe {
        let p = ffi::gosh_call(std::ptr::null(), 0);
        let value: serde_json::Value =
            serde_json::from_str(std::ffi::CStr::from_ptr(p).to_str().unwrap()).unwrap();
        assert_eq!(value["ok"], false);
        ffi::gosh_free(p);
        for data in [b"bad".as_slice(), br#"{"operation":"not real"}"#.as_slice()] {
            let p = ffi::gosh_call(data.as_ptr(), data.len());
            assert!(std::ffi::CStr::from_ptr(p)
                .to_str()
                .unwrap()
                .contains("\"ok\":false"));
            ffi::gosh_free(p);
        }
    }
}
#[test]
fn arbitrary_target_is_rejected_without_opening() {
    let fake = platform::Device {
        path: "/tmp/not-a-device".into(),
        name: "fake".into(),
        size: 512,
        identity: "fake".into(),
        mount_points: vec![],
        sector_size: 512,
    };
    assert!(platform::confirmed(&fake).is_err());
}

#[test]
fn readback_detects_corruption() {
    let (source_path, mut source) = fixture(&[7; 4096]);
    let (target_path, mut target) = fixture(&[]);
    let result = jobs::copy_verify(
        &mut source,
        &mut target,
        4096,
        true,
        1,
        &AtomicBool::new(false),
        &mut |phase, bytes, total, _| {
            if phase == "writing" && bytes == total {
                let mut corrupt = OpenOptions::new().write(true).open(&target_path).unwrap();
                corrupt.write_all(&[0]).unwrap();
            }
        },
    );
    assert_eq!(result.unwrap_err().kind, "verification");
    drop(source);
    drop(target);
    fs::remove_file(source_path).unwrap();
    fs::remove_file(target_path).unwrap();
}

#[test]
fn raw_sector_alignment_pads_only_the_tail_and_verifies_original_bytes() {
    let data = vec![211; 4 * 1024 * 1024 + 123];
    let (sp, mut source) = fixture(&data);
    let (tp, mut target) = fixture(&[]);
    jobs::copy_verify(
        &mut source,
        &mut target,
        data.len() as u64,
        true,
        4096,
        &AtomicBool::new(false),
        &mut |_, _, _, _| {},
    )
    .unwrap();
    let output = fs::read(&tp).unwrap();
    assert_eq!(output.len() % 4096, 0);
    assert_eq!(&output[..data.len()], data);
    assert!(output[data.len()..].iter().all(|b| *b == 0));
    drop(source);
    drop(target);
    fs::remove_file(sp).unwrap();
    fs::remove_file(tp).unwrap();
}
