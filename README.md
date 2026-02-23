# Dockerstrator

A generic Docker Compose orchestrator with an interactive interface and multi-select service options.

## Install

```bash
curl -sSL https://raw.githubusercontent.com/caduvarela/dockerstrator/master/install.sh | sh
```

That's it. The `dockerstrator` command is now available globally.

## Features

- **Agnostic**: Works with any folder structure containing `docker-compose.yml` files
- **Zero Configuration**: Automatically discovers all services recursively
- **Multi-select**: Choose which services to control with a clean interface
- **Full control**: Start, stop, restart, pull, and clean volumes
- **Status view**: See which services are UP or DOWN at a glance
- **Log streaming**: Tail logs from any service, Ctrl+C returns to menu
- **Keyboard-first**: Arrow keys and letter shortcuts for all actions
- **Configurable**: Set max search depth and exclude directories
- **Fast**: Compiled in Rust for instant execution

## Usage

```bash
dockerstrator [directory]
```

Defaults to the current directory if omitted.

Navigate with arrow keys or letter shortcuts, select services with SPACE, confirm with ENTER.

## Examples

```
services/
├── rabbitmq/
│   └── docker-compose.yml
├── minio/
│   └── docker-compose.yml
└── mailhog/
    └── docker-compose.yml
```

Run `dockerstrator` from this directory and select which services to manage.

## Build from Source

```bash
git clone https://github.com/caduvarela/dockerstrator.git
cd dockerstrator
cargo install --path .
```

Requires Rust 1.56+ and a C linker

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

MIT
