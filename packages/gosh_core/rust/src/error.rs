use serde::Serialize;
#[derive(Debug, thiserror::Error, Serialize, Clone)]
#[error("{message}")]
pub struct Error {
    pub kind: String,
    pub message: String,
}
pub type Result<T> = std::result::Result<T, Error>;
impl Error {
    pub fn new(kind: &str, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        let kind = if e.kind() == std::io::ErrorKind::PermissionDenied {
            "permission"
        } else {
            "io"
        };
        Self::new(kind, e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new("validation", e.to_string())
    }
}
