mod error;
mod ffi;
mod image;
mod jobs;
mod platform;
use error::{Error, Result};
use serde_json::{json, Value};
use std::path::Path;
fn text<'a>(request: &'a Value, key: &str) -> Result<&'a str> {
    request
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::new("validation", format!("Missing {key}.")))
}
fn dispatch(request: Value) -> Result<Value> {
    match text(&request, "operation")? {
        "inspect" => Ok(serde_json::to_value(image::inspect(Path::new(text(
            &request, "path",
        )?))?)?),
        "checksum" => Ok(
            json!({"checksum":image::checksum(Path::new(text(&request,"path")?),text(&request,"algorithm")?)?}),
        ),
        "devices" => Ok(json!({"devices":platform::devices()?})),
        "write" => Ok(
            json!({"id":jobs::start(serde_json::from_value(request.get("request").cloned().ok_or_else(||Error::new("validation","Missing write request."))?)?)?}),
        ),
        "poll" => Ok(serde_json::to_value(jobs::poll(
            request
                .get("id")
                .and_then(Value::as_u64)
                .ok_or_else(|| Error::new("validation", "Invalid operation id."))?,
        )?)?),
        "cancel" => {
            jobs::cancel(
                request
                    .get("id")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| Error::new("validation", "Invalid operation id."))?,
            )?;
            Ok(json!({}))
        }
        "busy" => Ok(json!({"busy":jobs::busy()})),
        "eject" => {
            if jobs::busy() {
                return Err(Error::new(
                    "busy",
                    "Wait for the write operation to finish.",
                ));
            }
            let d = platform::confirmed(&serde_json::from_value(request["device"].clone())?)?;
            platform::eject(&d)?;
            Ok(json!({}))
        }
        _ => Err(Error::new("validation", "Unknown operation.")),
    }
}
#[cfg(test)]
mod tests;
