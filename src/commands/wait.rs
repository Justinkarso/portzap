use crate::errors::Result;
use crate::output::OutputFormat;
use crate::process::WaitCondition;
use crate::scanner::create_scanner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct WaitOptions {
    pub port: u16,
    pub condition: WaitCondition,
    pub timeout_secs: u64,
    pub poll_interval_ms: u64,
    pub format: OutputFormat,
}

pub fn execute(opts: WaitOptions) -> Result<bool> {
    let scanner = create_scanner();
    let poll_interval = Duration::from_millis(opts.poll_interval_ms);
    let timeout = if opts.timeout_secs == 0 {
        None
    } else {
        Some(Duration::from_secs(opts.timeout_secs))
    };
    let start = Instant::now();

    // Handle Ctrl+C gracefully
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    signal_hook::flag::register(signal_hook::consts::SIGINT, r)
        .expect("failed to register SIGINT handler");
    let r2 = running.clone();
    signal_hook::flag::register(signal_hook::consts::SIGTERM, r2)
        .expect("failed to register SIGTERM handler");

    let state_label = match opts.condition {
        WaitCondition::Free => "free",
        WaitCondition::Occupied => "occupied",
    };

    eprintln!(
        "Waiting for port {} to become {} (timeout: {}, poll: {}ms)",
        opts.port,
        state_label,
        if opts.timeout_secs == 0 {
            "infinite".to_string()
        } else {
            format!("{}s", opts.timeout_secs)
        },
        opts.poll_interval_ms,
    );

    while running.load(Ordering::Relaxed) {
        let processes = scanner.find_processes_by_port(opts.port)?;
        let is_free = processes.is_empty();

        let condition_met = match opts.condition {
            WaitCondition::Free => is_free,
            WaitCondition::Occupied => !is_free,
        };

        if condition_met {
            let status = if is_free { "free" } else { "occupied" };
            match opts.format {
                OutputFormat::Json => {
                    println!(
                        r#"{{"port": {}, "status": "{}"}}"#,
                        opts.port, status
                    );
                }
                _ => {
                    eprintln!("Port {} is {}", opts.port, status);
                }
            }
            return Ok(true);
        }

        if let Some(t) = timeout {
            if start.elapsed() >= t {
                match opts.format {
                    OutputFormat::Json => {
                        println!(
                            r#"{{"port": {}, "status": "timeout"}}"#,
                            opts.port
                        );
                    }
                    _ => {
                        eprintln!("Timeout: port {} did not become {}", opts.port, state_label);
                    }
                }
                return Ok(false);
            }
        }

        thread::sleep(poll_interval);
    }

    // Interrupted by signal
    match opts.format {
        OutputFormat::Json => {
            println!(
                r#"{{"port": {}, "status": "timeout"}}"#,
                opts.port
            );
        }
        _ => {
            eprintln!("\nInterrupted.");
        }
    }
    Ok(false)
}
