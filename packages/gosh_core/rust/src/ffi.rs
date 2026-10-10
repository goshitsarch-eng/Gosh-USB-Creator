//! Two C symbols. Request memory is borrowed only during the call. Result memory
//! is allocated by Rust and must be released exactly once by gosh_free.
use crate::{dispatch, error::Error};
use serde_json::json;
use std::{
    ffi::{c_char, CString},
    panic::{catch_unwind, AssertUnwindSafe},
};
#[no_mangle]
pub unsafe extern "C" fn gosh_call(input: *const u8, length: usize) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if input.is_null() || length > 1024 * 1024 {
            return Err(Error::new("validation", "Invalid FFI request buffer."));
        }
        // SAFETY: ABI caller guarantees a readable length-byte slice for this
        // synchronous call. It remains borrowed and is never retained by Rust.
        let bytes = unsafe { std::slice::from_raw_parts(input, length) };
        dispatch(serde_json::from_slice(bytes)?)
    }))
    .unwrap_or_else(|_| {
        Err(Error::new(
            "internal",
            "The backend encountered an unexpected error.",
        ))
    });
    let envelope = match result {
        Ok(value) => json!({"ok":true,"value":value}),
        Err(error) => json!({"ok":false,"error":error}),
    };
    // JSON string encoding escapes embedded NUL characters.
    CString::new(envelope.to_string())
        .expect("JSON has no literal NUL")
        .into_raw()
}
#[no_mangle]
pub unsafe extern "C" fn gosh_free(output: *mut c_char) {
    if !output.is_null() {
        // SAFETY: ABI caller passes exactly one live pointer returned by
        // gosh_call, at most once. Allocator ownership stays with Rust.
        drop(unsafe { CString::from_raw(output) });
    }
}
