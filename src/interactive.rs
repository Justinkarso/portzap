use crate::process::ProcessInfo;
use dialoguer::MultiSelect;

/// Present processes to the user and let them pick which to kill.
pub fn select_processes(processes: &[ProcessInfo]) -> Vec<ProcessInfo> {
    if processes.is_empty() {
        return Vec::new();
    }

    let display_items: Vec<String> = processes
        .iter()
        .map(|p| {
            let cmd = p
                .command
                .as_deref()
                .map(|c| {
                    let truncated = if c.len() > 60 { &c[..60] } else { c };
                    format!(" ({})", truncated)
                })
                .unwrap_or_default();
            format!(
                "PID {:>6} | {:>5}/{} | {}{}",
                p.pid, p.port, p.protocol, p.name, cmd
            )
        })
        .collect();

    let defaults = vec![true; processes.len()];

    let selections = MultiSelect::new()
        .with_prompt("Select processes to kill (Space to toggle, Enter to confirm)")
        .items(&display_items)
        .defaults(&defaults)
        .interact();

    match selections {
        Ok(indices) => indices
            .into_iter()
            .map(|i| processes[i].clone())
            .collect(),
        Err(_) => {
            eprintln!("Selection cancelled.");
            Vec::new()
        }
    }
}
