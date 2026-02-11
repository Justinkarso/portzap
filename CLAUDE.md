# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                    # Dev build
cargo build --release          # Release build (LTO, stripped)
cargo test                     # Run all tests (unit + integration)
cargo test <test_name>         # Run a single test
cargo clippy -- -D warnings    # Lint (CI enforces zero warnings)
cargo fmt --check              # Check formatting (CI enforces)
cargo fmt                      # Auto-format
```

Integration tests use `ListenerGuard` (in `tests/helpers/mod.rs`) to bind real TCP ports, so tests exercise actual OS port scanning.

## Architecture

```
CLI parsing (main.rs, cli.rs)
    ↓
Command layer (commands/{kill,list,watch,free,wait,completions}.rs)
    ↓
PortScanner trait (scanner.rs)
    ↓
Platform implementations (platform/{macos,linux,windows}.rs)
    ↓
OS APIs (libproc on macOS, procfs on Linux)
```

**Key patterns:**
- `scanner::create_scanner()` is a factory that returns the platform-specific `Box<dyn PortScanner>` via conditional compilation
- `process.rs` holds shared types: `ProcessInfo`, `KillSignal`, `KillResult`, `PortSpec`, `WaitCondition`
- `output.rs` handles Table/JSON/Plain formatting — human messages go to stderr, structured data to stdout
- `killer.rs` implements graceful shutdown: SIGTERM → poll → SIGKILL escalation
- `cli.rs` defines clap structs; `main.rs` converts CLI enums to internal types (e.g., `convert_signal`, `convert_format`)
- Long-running commands (`watch`, `wait`) use `signal_hook` with `Arc<AtomicBool>` for SIGINT/SIGTERM handling
- The `tui.rs` module is a self-contained ratatui app with its own event loop, theming, and config persistence

**Config:** stored at `~/.config/portzap/config.toml` (theme, confirmation dialog, animation duration).

## NPM Distribution

The `npm/` directory contains platform-specific packages (`@portzap/{platform}-{arch}`) and a wrapper package. The release workflow (`.github/workflows/release.yml`) builds binaries, copies them into npm package dirs, and publishes to both npm and crates.io on git tags matching `v*`.

## CI

Runs on push/PR: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` on both macOS and Ubuntu.

## MSRV

Minimum supported Rust version: **1.74** (set in `Cargo.toml`).

## Windows

The Windows platform scanner (`platform/windows.rs`) and killer are stubs returning "not yet implemented" errors. macOS and Linux are fully implemented.
