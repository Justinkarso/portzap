use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum KillportError {
    #[error("port {0} is not in valid range (1-65535)")]
    InvalidPort(u32),

    #[error("invalid port range: {0}")]
    InvalidPortRange(String),

    #[error("no process found on port {port}")]
    NoProcessFound { port: u16 },

    #[error("failed to kill process {pid} ({name}): {reason}")]
    KillFailed {
        pid: u32,
        name: String,
        reason: String,
    },

    #[error("permission denied: cannot kill process {pid} ({name}). Try running with sudo")]
    PermissionDenied { pid: u32, name: String },

    #[error("platform error: {0}")]
    PlatformError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, KillportError>;
