# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-02-20

### Added
- Initial release of Dockerstrator
- Auto-discovery of docker-compose.yml files in subdirectories
- Interactive menu with arrow key navigation
- Multi-select interface for choosing services (using SPACE)
- Start, stop, restart operations for services
- Service status display
- Real-time log streaming with colored output
- Volume cleanup functionality
- Support for both `docker-compose` and `docker compose` commands
- Works with any docker-compose.yml folder structure

### Technical
- Built with Rust for performance
- Uses `inquire` library for terminal UI
- Uses `colored` library for terminal colors
- Cross-platform compatible
