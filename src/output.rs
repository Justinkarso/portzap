use crate::process::{KillResult, ProcessInfo};
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{ContentArrangement, Table};
use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Plain,
}

pub fn print_processes(processes: &[ProcessInfo], format: OutputFormat) {
    match format {
        OutputFormat::Table => print_process_table(processes),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(processes).unwrap_or_default()
            );
        }
        OutputFormat::Plain => {
            for p in processes {
                println!(
                    "{}\t{}\t{}\t{}",
                    p.pid, p.name, p.port, p.protocol
                );
            }
        }
    }
}

fn print_process_table(processes: &[ProcessInfo]) {
    if processes.is_empty() {
        return;
    }
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["PID", "Name", "Port", "Protocol", "Command"]);

    for p in processes {
        table.add_row(vec![
            p.pid.to_string(),
            p.name.clone(),
            p.port.to_string(),
            p.protocol.to_string(),
            p.command
                .as_deref()
                .map(truncate_command)
                .unwrap_or_else(|| "-".into()),
        ]);
    }
    println!("{table}");
}

pub fn print_kill_results(results: &[KillResult], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(results).unwrap_or_default()
            );
        }
        OutputFormat::Table | OutputFormat::Plain => {
            for r in results {
                if r.success {
                    eprintln!(
                        "{} Killed {} (PID {}) on port {}/{} [{}]",
                        "✓".green(),
                        r.process.name.bold(),
                        r.process.pid,
                        r.process.port,
                        r.process.protocol,
                        r.signal_sent.dimmed(),
                    );
                } else {
                    eprintln!(
                        "{} Failed to kill {} (PID {}): {}",
                        "✗".red(),
                        r.process.name.bold(),
                        r.process.pid,
                        r.error.as_deref().unwrap_or("unknown error"),
                    );
                }
            }
        }
    }
}

pub fn print_no_process(port: u16, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(r#"{{"port": {port}, "processes": []}}"#);
        }
        _ => {
            eprintln!("No processes found on port {port}");
        }
    }
}

fn truncate_command(s: &str) -> String {
    if s.len() > 80 {
        format!("{}...", &s[..77])
    } else {
        s.to_string()
    }
}
