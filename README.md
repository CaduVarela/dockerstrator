# Dockerstrator

A generic Docker Compose orchestrator with an interactive interface and multi-select service options.

## Install

```bash
curl -sSL https://raw.githubusercontent.com/caduvarela/dockerstrator/master/install.sh | sh
```

That's it. The `dockerstrator` command is now available globally.

## Features

- **Agnostic**: Works with any folder structure containing `docker-compose.yml` files
- **Zero Configuration**: Automatically discovers all services
- **Multi-select**: Choose which services to control with a clean interface
- **Fast**: Compiled in Rust for instant execution
- **Simple**: Single command installation

## Usage

```bash
dockerstrator
```

Navigate with arrow keys, select with SPACE, confirm with ENTER.

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
cargo build --release
cp target/release/dockerstrator ~/.local/bin/
```

Requires: Rust 1.56+, build-essential

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

MIT
