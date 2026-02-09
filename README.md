# portzap

A fast, cross-platform port management tool. Kill, list, and watch processes on network ports.

## Features

- **Kill processes**: Terminate processes running on specified ports
- **List ports**: View all listening ports or inspect specific ones
- **Watch ports**: Automatically kill any process that binds to watched ports
- **Interactive mode**: Select which processes to kill interactively
- **Cross-platform**: Works on macOS, Linux, and Windows
- **Graceful shutdown**: Sends SIGTERM first, escalates to SIGKILL if needed

## Installation

### Using Cargo (Recommended)

```bash
cargo install portzap
```

### Using npm

```bash
npm install -g portzap
```

### From source

```bash
git clone https://github.com/justinkarso/portzap
cd portzap
cargo install --path .
```

## Usage

### Kill processes on ports

```bash
# Kill process on port 3000
portzap 3000

# Kill processes on multiple ports
portzap 3000 8080 9090

# Kill processes on port range
portzap 3000-3010

# Interactive mode: choose which process to kill
portzap -i 3000

# Dry run: show what would be killed without killing
portzap --dry-run 3000
```

### List processes on ports

```bash
# List all listening ports
portzap list

# Show what's on port 3000
portzap list 3000
```

### Watch ports

```bash
# Watch port 3000 and auto-kill anything that binds to it
portzap watch 3000

# Watch multiple ports
portzap watch 3000 8080
```

## Options

- `-i, --interactive`: Interactive mode to select processes
- `--dry-run`: Show what would be killed without actually killing
- `-s, --signal`: Signal to send (term, kill, int, hup)
- `--no-graceful`: Skip graceful shutdown, send signal immediately
- `-t, --timeout`: Timeout for graceful shutdown (default: 5 seconds)
- `--format`: Output format (table, json, plain)

## Examples

```bash
# Kill development server on port 3000
portzap 3000

# List all ports in JSON format
portzap list --format json

# Watch port 8080 with 2-second poll interval
portzap watch 8080 --poll 2000

# Kill process on port 5000 interactively
portzap -i 5000
```

## License

MIT OR Apache-2.0
