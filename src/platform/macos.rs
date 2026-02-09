use crate::errors::{KillportError, Result};
use crate::process::{ProcessInfo, Protocol};
use crate::scanner::PortScanner;
use libproc::file_info::{pidfdinfo, ListFDs, ProcFDType};
use libproc::net_info::{SocketFDInfo, SocketInfoKind};
use libproc::proc_pid::{listpidinfo, name, pidpath};
use libproc::processes::{pids_by_type, ProcFilter};
use std::collections::HashMap;

pub struct MacosScanner;

impl MacosScanner {
    pub fn new() -> Self {
        Self
    }

    fn get_process_info(pid: i32, port: u16, protocol: Protocol) -> ProcessInfo {
        let proc_name = name(pid).unwrap_or_else(|_| "<unknown>".into());
        let command = pidpath(pid).ok();
        ProcessInfo {
            pid: pid as u32,
            name: proc_name,
            port,
            protocol,
            command,
            user: None,
        }
    }

    /// Extract local port and protocol from a socket's info, if applicable.
    fn extract_port_info(socket_info: &SocketFDInfo) -> Option<(u16, Protocol)> {
        let kind = socket_info.psi.soi_kind;
        if kind == SocketInfoKind::Tcp as i32 {
            let tcp = unsafe { socket_info.psi.soi_proto.pri_tcp };
            let port = u16::from_be(tcp.tcpsi_ini.insi_lport as u16);
            if port > 0 {
                return Some((port, Protocol::Tcp));
            }
        } else if kind == SocketInfoKind::In as i32 {
            // UDP sockets show up as SocketInfoKind::In
            let inp = unsafe { socket_info.psi.soi_proto.pri_in };
            let port = u16::from_be(inp.insi_lport as u16);
            if port > 0 {
                return Some((port, Protocol::Udp));
            }
        }
        None
    }

    /// Iterate all file descriptors of a process and collect port bindings.
    fn scan_process_fds(
        pid: i32,
        port_filter: Option<u16>,
    ) -> Vec<(u16, Protocol)> {
        let fds = match listpidinfo::<ListFDs>(pid, 256) {
            Ok(fds) => fds,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();

        for fd in &fds {
            if fd.proc_fdtype != ProcFDType::Socket as u32 {
                continue;
            }

            let socket_info = match pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd) {
                Ok(info) => info,
                Err(_) => continue,
            };

            if let Some((port, protocol)) = Self::extract_port_info(&socket_info) {
                if let Some(target) = port_filter {
                    if port == target {
                        results.push((port, protocol));
                        break; // Found the target port for this process
                    }
                } else {
                    results.push((port, protocol));
                }
            }
        }

        results
    }
}

impl PortScanner for MacosScanner {
    fn find_processes_by_port(&self, target_port: u16) -> Result<Vec<ProcessInfo>> {
        let pids = pids_by_type(ProcFilter::All)
            .map_err(|e| KillportError::PlatformError(format!("failed to list PIDs: {e}")))?;

        let mut results: HashMap<u32, ProcessInfo> = HashMap::new();

        for pid in pids {
            if pid == 0 {
                continue;
            }

            let matches = Self::scan_process_fds(pid as i32, Some(target_port));
            for (port, protocol) in matches {
                let pid_u32 = pid;
                results
                    .entry(pid_u32)
                    .or_insert_with(|| Self::get_process_info(pid as i32, port, protocol));
            }
        }

        Ok(results.into_values().collect())
    }

    fn find_all_listening(&self) -> Result<Vec<ProcessInfo>> {
        let pids = pids_by_type(ProcFilter::All)
            .map_err(|e| KillportError::PlatformError(format!("failed to list PIDs: {e}")))?;

        let mut results: Vec<ProcessInfo> = Vec::new();
        let mut seen: HashMap<(u32, u16, u8), bool> = HashMap::new();

        for pid in pids {
            if pid == 0 {
                continue;
            }

            let matches = Self::scan_process_fds(pid as i32, None);
            for (port, protocol) in matches {
                let key = (
                    pid,
                    port,
                    match protocol {
                        Protocol::Tcp => 0,
                        Protocol::Udp => 1,
                    },
                );
                if seen.contains_key(&key) {
                    continue;
                }
                seen.insert(key, true);
                results.push(Self::get_process_info(pid as i32, port, protocol));
            }
        }

        // Sort by port, then PID
        results.sort_by_key(|p| (p.port, p.pid));
        Ok(results)
    }
}
