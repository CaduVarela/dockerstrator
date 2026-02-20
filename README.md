# Dockerstrator

A generic Docker Compose orchestrator with an interactive interface and multi-select service options.

## Features

- **Agnostic**: Works with any folder structure containing `docker-compose.yml` files
- **Zero Configuration**: Automatically discovers all services
- **Multi-select**: Choose which services to control with a clean interface
- **Fast**: Compiled in Rust for instant execution
- **Professional**: Clean and intuitive interface

## Quick Installation

### Option 1: Pre-compiled Binary (Recommended)

If you already have the compiled executable:

```bash
cp orchestrator/target/release/dockerstrator ~/.local/bin/
dockerstrator
```

### Option 2: Compile from Source

Make sure you have Rust installed ([rustup.rs](https://rustup.rs/)) and build-essential:

```bash
# Ubuntu/Debian
sudo apt-get install -y build-essential

# Compile
cd orchestrator
cargo build --release

# The executable will be at: target/release/dockerstrator
```

Or use the provided build script:

```bash
./build.sh
```

## Usage

Run from the root folder where your services are located:

```bash
cd services/
dockerstrator
```

Or create a permanent alias:

```bash
echo "alias dockerstrator='~/.local/bin/dockerstrator'" >> ~/.bashrc
source ~/.bashrc
```

## Expected Folder Structure

```
services/
├── rabbitmq/
│   └── docker-compose.yml
├── minio/
│   └── docker-compose.yml
├── mailhog/
│   └── docker-compose.yml
└── orchestrator/
    └── dockerstrator (executable)
```

## Available Operations

| Operation | Description |
|-----------|-------------|
| **Start** | Brings up selected services |
| **Stop** | Stops all services |
| **Restart** | Restarts selected services |
| **Status** | Shows status of each service |
| **Logs** | Streams logs in real-time |
| **Cleanup** | Removes volumes of selected services |
| **Exit** | Closes the application |

### Example Usage

```bash
$ dockerstrator
Docker Services Orchestrator

Services found: 3

What would you like to do?
  > Start
    Stop
    Restart
    Status
    Logs
    Cleanup
    Exit
```

**Controls:**
- Arrow keys ↑↓ to navigate menu
- ENTER to select an option
- SPACE to mark/unmark services (multi-select)
- ENTER to confirm selection

## Requirements

- **Runtime**: Docker or Docker Compose
- **Permissions**: Access to Docker (docker group or sudo)
- **Compilation** (optional): Rust 1.56+ and build-essential

## Development

The application is completely agnostic and works with any `docker-compose.yml`.

### Project Structure

- `src/main.rs` - Main application logic
- `Cargo.toml` - Dependencies and metadata
- `build.sh` - Build script

### Adding Features

Edit `src/main.rs` and recompile:

```bash
cargo build --release
```

For contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md)

## License

MIT

## Author

Carlos Varela
