# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-30

### Added
- **Core Architecture**: Initial release of GameDoc with an immutable `Facts` pipeline.
- **System Collector**: Safely detects OS details, hostname, memory, Wayland/X11 session type, and architecture.
- **GPU Collector**: Accurately detects GPUs via `lspci`, driver info via `glxinfo`, `vulkaninfo`, and `nvidia-smi`.
- **Steam Collector**: Full support for detecting Native, Flatpak, and Snap Steam installations, including uninitialized states. Supports Official Proton and Custom Proton (e.g., GE-Proton) discovery.
- **Renderers**:
  - `Terminal`: Colorful, clean output with clear categorization and statuses.
  - `Markdown`: Automatically formats outputs for GitHub issues.
  - `JSON`: Highly structured, `snake_case`, stable schema version 2.
- **Security & Privacy**: Read-only assertion. 100% telemetry free. No shell evaluations (`sh -c` is banned). Includes a `--share` flag to sanitize usernames.

### Changed
- Built with a strict `clippy.toml` to ban mutations and guarantee deterministic outcomes.
