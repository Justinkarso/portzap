use clap::{Args, Parser, Subcommand, ValueEnum};

/// A fast, cross-platform port management tool.
/// Kill, list, and watch processes on network ports.
#[derive(Parser, Debug)]
#[command(
    name = "portzap",
    version,
    about,
    long_about = None,
    arg_required_else_help = true,
    after_help = "\x1b[1mExamples:\x1b[0m
  portzap 3000              Kill process on port 3000
  portzap 3000 8080 9090    Kill processes on multiple ports
  portzap 3000-3010         Kill processes on port range
  portzap -i 3000           Interactive mode: choose which to kill
  portzap --dry-run 3000    Show what would be killed
  portzap list              List all listening ports
  portzap list 3000         Show what's on port 3000
  portzap watch 3000        Watch and auto-kill anything on port 3000"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Ports to kill processes on (default action).
    /// Supports single ports (3000) and ranges (3000-3010).
    #[arg(value_name = "PORTS", num_args = 1..)]
    pub ports: Vec<String>,

    /// Signal to send
    #[arg(short, long, value_enum, default_value_t = Signal::Term)]
    pub signal: Signal,

    /// Disable graceful shutdown (skip SIGTERM, send signal immediately)
    #[arg(long)]
    pub no_graceful: bool,

    /// Timeout in seconds for graceful shutdown before escalating to SIGKILL
    #[arg(short, long, default_value_t = 5)]
    pub timeout: u64,

    /// Show what would be killed without actually killing
    #[arg(long)]
    pub dry_run: bool,

    /// Interactive mode: select which processes to kill
    #[arg(short, long)]
    pub interactive: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Table, global = true)]
    pub format: Format,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Kill processes on specified ports (same as default behavior)
    Kill(KillArgs),

    /// List processes on ports (or all listening ports if none specified)
    List(ListArgs),

    /// Watch ports and auto-kill anything that binds to them
    Watch(WatchArgs),
}

#[derive(Args, Debug)]
pub struct KillArgs {
    /// Ports to kill processes on. Supports ranges like 3000-3010.
    #[arg(value_name = "PORTS", num_args = 1.., required = true)]
    pub ports: Vec<String>,

    /// Signal to send
    #[arg(short, long, value_enum, default_value_t = Signal::Term)]
    pub signal: Signal,

    /// Disable graceful shutdown
    #[arg(long)]
    pub no_graceful: bool,

    /// Timeout in seconds for graceful shutdown before escalating to SIGKILL
    #[arg(short, long, default_value_t = 5)]
    pub timeout: u64,

    /// Show what would be killed without actually killing
    #[arg(long)]
    pub dry_run: bool,

    /// Interactive mode: select which processes to kill
    #[arg(short, long)]
    pub interactive: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Table)]
    pub format: Format,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Ports to inspect. If omitted, lists all listening ports.
    #[arg(value_name = "PORTS")]
    pub ports: Vec<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Table)]
    pub format: Format,
}

#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Ports to watch. Supports ranges like 3000-3010.
    #[arg(value_name = "PORTS", num_args = 1.., required = true)]
    pub ports: Vec<String>,

    /// Signal to send to new processes
    #[arg(short, long, value_enum, default_value_t = Signal::Term)]
    pub signal: Signal,

    /// Disable graceful shutdown
    #[arg(long)]
    pub no_graceful: bool,

    /// Graceful timeout in seconds
    #[arg(short, long, default_value_t = 5)]
    pub timeout: u64,

    /// Poll interval in milliseconds
    #[arg(long, default_value_t = 1000)]
    pub poll: u64,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Table)]
    pub format: Format,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Signal {
    Term,
    Kill,
    Int,
    Hup,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Table,
    Json,
    Plain,
}
