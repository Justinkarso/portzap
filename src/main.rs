mod cli;
mod commands;
mod config;
mod errors;
mod interactive;
mod killer;
mod output;
mod platform;
mod process;
mod scanner;
mod theme;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands, Format, Signal, WaitUntil};
use output::OutputFormat;
use process::{KillSignal, PortSpec, WaitCondition};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let format = convert_format(cli.format);

    match cli.command {
        Some(Commands::Kill(args)) => {
            let ports = parse_ports(&args.ports)?;
            let success = commands::kill::execute(commands::kill::KillOptions {
                ports,
                signal: convert_signal(args.signal),
                graceful: !args.no_graceful,
                graceful_timeout_secs: args.timeout,
                dry_run: args.dry_run,
                interactive: args.interactive,
                format: convert_format(args.format),
            })?;
            if !success {
                std::process::exit(1);
            }
        }

        Some(Commands::List(args)) => {
            let ports = parse_ports(&args.ports)?;
            commands::list::execute(commands::list::ListOptions {
                ports,
                format: convert_format(args.format),
            })?;
        }

        Some(Commands::Free(args)) => {
            let result = commands::free::execute(commands::free::FreeOptions {
                start: args.port,
                max: args.max,
                format: convert_format(args.format),
            })?;
            if result.is_none() {
                std::process::exit(1);
            }
        }

        Some(Commands::Wait(args)) => {
            let success = commands::wait::execute(commands::wait::WaitOptions {
                port: args.port,
                condition: convert_wait_until(args.until),
                timeout_secs: args.timeout,
                poll_interval_ms: args.poll,
                format: convert_format(args.format),
            })?;
            if !success {
                std::process::exit(1);
            }
        }

        Some(Commands::Completions(args)) => {
            commands::completions::execute(args.shell);
        }

        Some(Commands::Gui) => {
            tui::run()?;
        }

        Some(Commands::Watch(args)) => {
            let ports = parse_ports(&args.ports)?;
            commands::watch::execute(commands::watch::WatchOptions {
                ports,
                signal: convert_signal(args.signal),
                graceful: !args.no_graceful,
                graceful_timeout_secs: args.timeout,
                poll_interval_ms: args.poll,
                format: convert_format(args.format),
            })?;
        }

        None => {
            // Default action: kill (bare `portzap 3000 8080`)
            if cli.ports.is_empty() {
                // arg_required_else_help should prevent this
                return Ok(());
            }

            let ports = parse_ports(&cli.ports)?;
            let success = commands::kill::execute(commands::kill::KillOptions {
                ports,
                signal: convert_signal(cli.signal),
                graceful: !cli.no_graceful,
                graceful_timeout_secs: cli.timeout,
                dry_run: cli.dry_run,
                interactive: cli.interactive,
                format,
            })?;
            if !success {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn parse_ports(raw: &[String]) -> Result<Vec<PortSpec>> {
    raw.iter()
        .map(|s| PortSpec::parse(s).with_context(|| format!("invalid port: '{s}'")))
        .collect()
}

fn convert_signal(s: Signal) -> KillSignal {
    match s {
        Signal::Term => KillSignal::Term,
        Signal::Kill => KillSignal::Kill,
        Signal::Int => KillSignal::Int,
        Signal::Hup => KillSignal::Hup,
    }
}

fn convert_format(f: Format) -> OutputFormat {
    match f {
        Format::Table => OutputFormat::Table,
        Format::Json => OutputFormat::Json,
        Format::Plain => OutputFormat::Plain,
    }
}

fn convert_wait_until(w: WaitUntil) -> WaitCondition {
    match w {
        WaitUntil::Down => WaitCondition::Free,
        WaitUntil::Up => WaitCondition::Occupied,
    }
}
