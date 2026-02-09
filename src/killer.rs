use crate::process::{KillResult, KillSignal, ProcessInfo};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct KillConfig {
    pub signal: KillSignal,
    pub graceful: bool,
    pub graceful_timeout: Duration,
    pub dry_run: bool,
}

impl Default for KillConfig {
    fn default() -> Self {
        Self {
            signal: KillSignal::Term,
            graceful: true,
            graceful_timeout: Duration::from_secs(5),
            dry_run: false,
        }
    }
}

pub fn kill_process(process: &ProcessInfo, config: &KillConfig) -> KillResult {
    if config.dry_run {
        return KillResult {
            process: process.clone(),
            success: true,
            signal_sent: format!("{} (dry-run)", config.signal),
            error: None,
        };
    }

    if config.graceful {
        graceful_kill(process, config)
    } else {
        force_kill(process, config)
    }
}

fn graceful_kill(process: &ProcessInfo, config: &KillConfig) -> KillResult {
    // Step 1: Send SIGTERM
    if let Err(e) = send_signal(process.pid, KillSignal::Term) {
        return KillResult {
            process: process.clone(),
            success: false,
            signal_sent: KillSignal::Term.to_string(),
            error: Some(e),
        };
    }

    // Step 2: Poll until process exits or timeout
    let start = Instant::now();
    let poll_interval = Duration::from_millis(100);

    while start.elapsed() < config.graceful_timeout {
        if !is_process_alive(process.pid) {
            return KillResult {
                process: process.clone(),
                success: true,
                signal_sent: KillSignal::Term.to_string(),
                error: None,
            };
        }
        thread::sleep(poll_interval);
    }

    // Step 3: Escalate to SIGKILL
    match send_signal(process.pid, KillSignal::Kill) {
        Ok(()) => {
            // Give it a moment to actually die
            thread::sleep(Duration::from_millis(100));
            KillResult {
                process: process.clone(),
                success: true,
                signal_sent: format!("{} -> {}", KillSignal::Term, KillSignal::Kill),
                error: None,
            }
        }
        Err(e) => KillResult {
            process: process.clone(),
            success: false,
            signal_sent: KillSignal::Kill.to_string(),
            error: Some(e),
        },
    }
}

fn force_kill(process: &ProcessInfo, config: &KillConfig) -> KillResult {
    match send_signal(process.pid, config.signal) {
        Ok(()) => KillResult {
            process: process.clone(),
            success: true,
            signal_sent: config.signal.to_string(),
            error: None,
        },
        Err(e) => KillResult {
            process: process.clone(),
            success: false,
            signal_sent: config.signal.to_string(),
            error: Some(e),
        },
    }
}

#[cfg(unix)]
fn send_signal(pid: u32, signal: KillSignal) -> std::result::Result<(), String> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    let nix_signal = match signal {
        KillSignal::Term => Signal::SIGTERM,
        KillSignal::Kill => Signal::SIGKILL,
        KillSignal::Int => Signal::SIGINT,
        KillSignal::Hup => Signal::SIGHUP,
    };

    signal::kill(Pid::from_raw(pid as i32), nix_signal).map_err(|e| {
        if e == nix::errno::Errno::EPERM {
            "permission denied. Try running with sudo".to_string()
        } else if e == nix::errno::Errno::ESRCH {
            "process no longer exists".to_string()
        } else {
            e.to_string()
        }
    })
}

#[cfg(windows)]
fn send_signal(pid: u32, _signal: KillSignal) -> std::result::Result<(), String> {
    Err("Windows kill not yet implemented".into())
}

#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    use nix::sys::signal;
    use nix::unistd::Pid;
    // Signal 0 checks if process exists without signaling it
    signal::kill(Pid::from_raw(pid as i32), None).is_ok()
}

#[cfg(windows)]
fn is_process_alive(_pid: u32) -> bool {
    false
}
