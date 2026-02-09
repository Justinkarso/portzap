use crate::errors::Result;
use crate::interactive;
use crate::killer::{self, KillConfig};
use crate::output::{self, OutputFormat};
use crate::process::{KillSignal, PortSpec};
use crate::scanner::create_scanner;
use std::time::Duration;

pub struct KillOptions {
    pub ports: Vec<PortSpec>,
    pub signal: KillSignal,
    pub graceful: bool,
    pub graceful_timeout_secs: u64,
    pub dry_run: bool,
    pub interactive: bool,
    pub format: OutputFormat,
}

pub fn execute(opts: KillOptions) -> Result<bool> {
    let scanner = create_scanner();
    let kill_config = KillConfig {
        signal: opts.signal,
        graceful: opts.graceful,
        graceful_timeout: Duration::from_secs(opts.graceful_timeout_secs),
        dry_run: opts.dry_run,
    };

    let ports: Vec<u16> = opts.ports.iter().flat_map(|ps| ps.expand()).collect();
    let mut all_success = true;

    for port in &ports {
        let mut processes = scanner.find_processes_by_port(*port)?;

        if processes.is_empty() {
            output::print_no_process(*port, opts.format);
            continue;
        }

        // In interactive mode, let the user pick
        if opts.interactive {
            // Always show what we found first
            if opts.format == OutputFormat::Table {
                output::print_processes(&processes, OutputFormat::Table);
            }
            processes = interactive::select_processes(&processes);
            if processes.is_empty() {
                continue;
            }
        }

        // Kill each process
        let mut results = Vec::new();
        for process in &processes {
            let result = killer::kill_process(process, &kill_config);
            if !result.success {
                all_success = false;
            }
            results.push(result);
        }

        output::print_kill_results(&results, opts.format);
    }

    Ok(all_success)
}
