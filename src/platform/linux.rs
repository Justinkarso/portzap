use crate::errors::{KillportError, Result};
use crate::process::{ProcessInfo, Protocol};
use crate::scanner::PortScanner;
use std::collections::HashMap;

pub struct LinuxScanner;

impl LinuxScanner {
    pub fn new() -> Self {
        Self
    }

    /// Read /proc/net/{tcp,tcp6,udp,udp6} and find inodes for the target port.
    fn find_inodes_for_port(target_port: u16) -> Result<HashMap<u64, Protocol>> {
        let mut inodes: HashMap<u64, Protocol> = HashMap::new();

        if let Ok(tcp) = procfs::net::tcp() {
            for entry in tcp {
                if entry.local_address.port() == target_port {
                    inodes.insert(entry.inode, Protocol::Tcp);
                }
            }
        }
        if let Ok(tcp6) = procfs::net::tcp6() {
            for entry in tcp6 {
                if entry.local_address.port() == target_port {
                    inodes.insert(entry.inode, Protocol::Tcp);
                }
            }
        }
        if let Ok(udp) = procfs::net::udp() {
            for entry in udp {
                if entry.local_address.port() == target_port {
                    inodes.insert(entry.inode, Protocol::Udp);
                }
            }
        }
        if let Ok(udp6) = procfs::net::udp6() {
            for entry in udp6 {
                if entry.local_address.port() == target_port {
                    inodes.insert(entry.inode, Protocol::Udp);
                }
            }
        }

        Ok(inodes)
    }

    /// Walk all processes and find which ones own any of the target inodes.
    fn find_processes_by_inodes(
        inodes: &HashMap<u64, Protocol>,
        target_port: u16,
    ) -> Result<Vec<ProcessInfo>> {
        let mut results = Vec::new();

        let all_procs = procfs::process::all_processes()
            .map_err(|e| KillportError::PlatformError(format!("failed to read /proc: {e}")))?;

        for proc_result in all_procs {
            let proc_entry = match proc_result {
                Ok(p) => p,
                Err(_) => continue,
            };

            let fds = match proc_entry.fd() {
                Ok(fds) => fds,
                Err(_) => continue,
            };

            for fd_result in fds {
                let fd_info = match fd_result {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                if let procfs::process::FDTarget::Socket(inode) = fd_info.target {
                    if let Some(&protocol) = inodes.get(&inode) {
                        let pid = proc_entry.pid() as u32;
                        let proc_name = proc_entry
                            .stat()
                            .map(|s| s.comm.clone())
                            .unwrap_or_else(|_| "<unknown>".into());
                        let command = proc_entry.cmdline().ok().map(|parts| parts.join(" "));

                        results.push(ProcessInfo {
                            pid,
                            name: proc_name,
                            port: target_port,
                            protocol,
                            command,
                            user: None,
                        });
                        break; // Found this PID, move to next process
                    }
                }
            }
        }

        Ok(results)
    }

    /// Collect all inodes with their port and protocol from /proc/net/*.
    fn all_listening_inodes() -> Result<HashMap<u64, (u16, Protocol)>> {
        let mut inode_map: HashMap<u64, (u16, Protocol)> = HashMap::new();

        if let Ok(tcp) = procfs::net::tcp() {
            for entry in tcp {
                let port = entry.local_address.port();
                if port > 0 {
                    inode_map.insert(entry.inode, (port, Protocol::Tcp));
                }
            }
        }
        if let Ok(tcp6) = procfs::net::tcp6() {
            for entry in tcp6 {
                let port = entry.local_address.port();
                if port > 0 {
                    inode_map.insert(entry.inode, (port, Protocol::Tcp));
                }
            }
        }
        if let Ok(udp) = procfs::net::udp() {
            for entry in udp {
                let port = entry.local_address.port();
                if port > 0 {
                    inode_map.insert(entry.inode, (port, Protocol::Udp));
                }
            }
        }
        if let Ok(udp6) = procfs::net::udp6() {
            for entry in udp6 {
                let port = entry.local_address.port();
                if port > 0 {
                    inode_map.insert(entry.inode, (port, Protocol::Udp));
                }
            }
        }

        Ok(inode_map)
    }
}

impl PortScanner for LinuxScanner {
    fn find_processes_by_port(&self, port: u16) -> Result<Vec<ProcessInfo>> {
        let inodes = Self::find_inodes_for_port(port)?;
        if inodes.is_empty() {
            return Ok(Vec::new());
        }
        Self::find_processes_by_inodes(&inodes, port)
    }

    fn find_all_listening(&self) -> Result<Vec<ProcessInfo>> {
        let inode_map = Self::all_listening_inodes()?;
        if inode_map.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        let all_procs = procfs::process::all_processes()
            .map_err(|e| KillportError::PlatformError(format!("failed to read /proc: {e}")))?;

        for proc_result in all_procs {
            let proc_entry = match proc_result {
                Ok(p) => p,
                Err(_) => continue,
            };

            let fds = match proc_entry.fd() {
                Ok(fds) => fds,
                Err(_) => continue,
            };

            for fd_result in fds {
                let fd_info = match fd_result {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                if let procfs::process::FDTarget::Socket(inode) = fd_info.target {
                    if let Some(&(port, protocol)) = inode_map.get(&inode) {
                        let pid = proc_entry.pid() as u32;
                        let proc_name = proc_entry
                            .stat()
                            .map(|s| s.comm.clone())
                            .unwrap_or_else(|_| "<unknown>".into());
                        let command = proc_entry.cmdline().ok().map(|parts| parts.join(" "));

                        results.push(ProcessInfo {
                            pid,
                            name: proc_name,
                            port,
                            protocol,
                            command,
                            user: None,
                        });
                    }
                }
            }
        }

        results.sort_by_key(|p| (p.port, p.pid));
        Ok(results)
    }
}
