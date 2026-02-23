# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-02-22

### Added
- Alternate terminal screen to avoid output pollution (tig-like experience)
- Arrow key navigation alongside letter shortcuts in all menus
- Keyboard shortcuts for all main menu items
- Settings menu with configuration persistence (`~/.config/dockerstrator/config.toml`)
- Configurable max search depth (default: 7 levels)
- Configurable excluded directories list
- Toggle between `docker compose` and `docker-compose` commands
- Multi-select support for stop operations (same as start and restart)
- Compact status display (`service: UP / service: DOWN`)
- Ctrl+C on log streaming returns to menu instead of exiting the program
- Y/N selection menu for destructive actions (stop, restart, cleanup)
- Help message shown in service selection menus
- Optional directory argument (`dockerstrator [path]`, defaults to current directory)

### Changed
- Stop services now pre-filters to show only currently running services
- Back/quit keybind changed from `b` to `q`
- Codebase split into modules: `config`, `docker`, `ops`, `service`, `ui`

### Fixed
- Menu display glitch when entering alternate terminal
- Header removed unintentionally after UI clear

### Performance
- Service status scraping optimized for faster display

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
