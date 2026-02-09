use crate::errors::{KillportError, Result};
use crate::process::ProcessInfo;
use crate::scanner::PortScanner;

pub struct WindowsScanner;

impl WindowsScanner {
    pub fn new() -> Self {
        Self
    }
}

impl PortScanner for WindowsScanner {
    fn find_processes_by_port(&self, _port: u16) -> Result<Vec<ProcessInfo>> {
        Err(KillportError::PlatformError(
            "Windows support is not yet implemented. Contributions welcome!".into(),
        ))
    }

    fn find_all_listening(&self) -> Result<Vec<ProcessInfo>> {
        Err(KillportError::PlatformError(
            "Windows support is not yet implemented. Contributions welcome!".into(),
        ))
    }
}
