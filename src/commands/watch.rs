use crate::errors::Result;
use crate::killer::{self, KillConfig};
use crate::output::{self, OutputFormat};
use crate::process::{KillSignal, PortSpec};
use crate::scanner::create_scanner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct WatchOptions {
    pub ports: Vec<PortSpec>,
    pub signal: KillSignal,
    pub graceful: bool,
    pub graceful_timeout_secs: u64,
    pub poll_interval_ms: u64,
    pub format: OutputFormat,
}

pub fn execute(opts: WatchOptions) -> Result<()> {
    let scanner = create_scanner();
    let kill_config = KillConfig {
        signal: opts.signal,
        graceful: opts.graceful,
        graceful_timeout: Duration::from_secs(opts.graceful_timeout_secs),
        dry_run: false,
    };
    let ports: Vec<u16> = opts.ports.iter().flat_map(|ps| ps.expand()).collect();
    let poll_interval = Duration::from_millis(opts.poll_interval_ms);

    // Handle Ctrl+C gracefully
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    signal_hook::flag::register(signal_hook::consts::SIGINT, r)
        .expect("failed to register SIGINT handler");
    let r2 = running.clone();
    signal_hook::flag::register(signal_hook::consts::SIGTERM, r2)
        .expect("failed to register SIGTERM handler");

    eprintln!(
        "Watching port{} {} (poll every {}ms, Ctrl+C to stop)",
        if ports.len() > 1 { "s" } else { "" },
        ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        opts.poll_interval_ms
    );

    while running.load(Ordering::Relaxed) {
        for port in &ports {
            let processes = scanner.find_processes_by_port(*port)?;
            for process in &processes {
                let result = killer::kill_process(process, &kill_config);
                output::print_kill_results(&[result], opts.format);
            }
        }
        thread::sleep(poll_interval);
    }

    eprintln!("\nWatch mode stopped.");
    Ok(())
}
