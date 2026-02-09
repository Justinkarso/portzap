use crate::errors::Result;
use crate::output::{self, OutputFormat};
use crate::process::PortSpec;
use crate::scanner::create_scanner;

pub struct ListOptions {
    pub ports: Vec<PortSpec>,
    pub format: OutputFormat,
}

pub fn execute(opts: ListOptions) -> Result<()> {
    let scanner = create_scanner();

    if opts.ports.is_empty() {
        // List ALL listening ports
        let processes = scanner.find_all_listening()?;
        if processes.is_empty() {
            eprintln!("No listening processes found");
        } else {
            output::print_processes(&processes, opts.format);
        }
    } else {
        let ports: Vec<u16> = opts.ports.iter().flat_map(|ps| ps.expand()).collect();
        for port in &ports {
            let processes = scanner.find_processes_by_port(*port)?;
            if processes.is_empty() {
                output::print_no_process(*port, opts.format);
            } else {
                output::print_processes(&processes, opts.format);
            }
        }
    }

    Ok(())
}
