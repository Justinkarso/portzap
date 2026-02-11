use crate::errors::Result;
use crate::output::OutputFormat;
use crate::scanner::create_scanner;

pub struct FreeOptions {
    pub start: u16,
    pub max: u16,
    pub format: OutputFormat,
}

pub fn execute(opts: FreeOptions) -> Result<Option<u16>> {
    let scanner = create_scanner();

    for port in opts.start..=opts.max {
        let processes = scanner.find_processes_by_port(port)?;
        if processes.is_empty() {
            match opts.format {
                OutputFormat::Json => {
                    println!(r#"{{"port": {port}}}"#);
                }
                _ => {
                    println!("{port}");
                }
            }
            return Ok(Some(port));
        }
    }

    // No free port found
    match opts.format {
        OutputFormat::Json => {
            println!(
                r#"{{"port": null, "error": "no free port found in range {}..={}"}}"#,
                opts.start, opts.max
            );
        }
        _ => {
            eprintln!(
                "No free port found in range {}..={}",
                opts.start, opts.max
            );
        }
    }
    Ok(None)
}
