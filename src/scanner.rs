use crate::errors::Result;
use crate::process::ProcessInfo;

pub trait PortScanner {
    /// Find all processes bound to the given port.
    fn find_processes_by_port(&self, port: u16) -> Result<Vec<ProcessInfo>>;

    /// Find all processes currently listening on any port.
    fn find_all_listening(&self) -> Result<Vec<ProcessInfo>>;
}

pub fn create_scanner() -> Box<dyn PortScanner> {
    #[cfg(target_os = "macos")]
    {
        Box::new(crate::platform::macos::MacosScanner::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(crate::platform::linux::LinuxScanner::new())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(crate::platform::windows::WindowsScanner::new())
    }
}
