# Changelog

<<<<<<< HEAD
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
=======
All notable changes are documented here. This project follows Semantic
Versioning after the initial public release.

## [Unreleased]

## [0.3.0] - 2026-08-01

### Added

- **Gaming Score hero**: Score is now the first thing you see — 30-block progress
  bar followed immediately by the verdict on the next line.
- **5-tier verdict system**: `Excellent Linux Gaming Environment` (95–100),
  `Ready for Linux Gaming` (80–94), `Gaming Ready — Minor Issues` (60–79),
  `Needs Attention` (40–59), `Gaming Environment Has Critical Issues` (0–39).
  Verdict is entirely score-driven; no contradictory status messages.
- **Kernel row** in the System section of the terminal dashboard.
- **Score test**: `score_100_maps_to_excellent_verdict` unit test.

### Changed

- Section names renamed: `System Profile` → `System`, `Gaming Stack` → `Gaming`
  for brevity and consistency with established tool conventions.
- Label column width reduced from 18 to 12 characters — better rhythm on
  standard 80-column terminals.
- Divider length reduced from 60 to 46 characters — cleaner on narrow terminals.
- Score bar widened to 30 blocks and color thresholds changed: green ≥ 80,
  yellow 60–79, red < 60.
- Footer spacing: `183 ms  ·  N facts  ·  N issues  ·  N skipped`.
- Recommendations now display a `Reason` label before the explanation and a
  `Command` label before the fix.
- Status labels for missing tools changed from `Missing` → `Not found`.
- Demo report now includes Kernel, uses clean driver string `4.6.0 NVIDIA 575.xx`,
  and uses boolean facts for VULKAN_AVAILABLE, GAMESCOPE_INSTALLATION, and
  GAMEMODE_DAEMON_RUNNING.
- `gaming_status()` return values align with the 5-tier verdict system.
- Golden snapshots regenerated for all 6 scenarios.
- Demo screenshot (`assets/demo.png`) regenerated: Catppuccin Mocha palette,
  JetBrains Mono font, rounded corners, drop shadow.


### Changed

- **Metadata:** Improved `Cargo.toml` with `repository`, `homepage`, `documentation`, updated `keywords` and `categories` for crates.io discoverability.
- **Documentation:** Fixed docs.rs compatibility warnings and improved library-level documentation presentation.

## [0.2.0] - 2026-07-31
### Added

- Structured `Recommendation` type with platform-specific instructions, difficulty, estimated time, and upstream documentation links.
- Compile-time severity registration with exhaustive rule mapping.
- `gamedoc.v2` JSON schema and `schema_url` injected during JSON rendering.
- Expanded privacy redaction to scrub build-host substrings (e.g. `user@host`) and the `$USER` environment variable.
- Distro-aware recommendations that pick the correct package manager command based on `distro.family`.
- Software Supply Chain CI: added `cargo-audit`, `cargo-deny`, `cosign` keyless signing, and CycloneDX SBOM generation.
- GitHub Actions versions pinned by SHA.
- `docs/collector-howto.md` guide for adding new checks.
- `MAINTAINERS.md` file.
- `BENCHMARKS.md` file documenting execution times.

- `DistroCollector`: reads `/etc/os-release`, emits `distro.name`, `distro.id`,
  `distro.version_id`, `distro.pretty_name`, and `distro.family`.
- `CpuCollector`: reads `/proc/cpuinfo`, emits vendor, model, clock speed,
  virtualization flags, logical core count, and physical core count.
  Also emits `cpu.architecture` via `uname -m`.
- `GameModeCollector`: expanded to probe the running daemon (`gamemoded -s`),
  detect the process with `pgrep gamemoded`, and check library paths.
- `MangoHudCollector`: expanded to detect global `/etc/MangoHud.conf`, user
  `~/.config/MangoHud/MangoHud.conf`, and environment variable evidence.
- `Severity` enum expanded to five levels: `Critical`, `High`, `Medium`,
  `Low`, and `Informational`.
- `Severity` and `FactConfidence` now implement `Display` (lowercase labels
  consistent with the JSON serde output); markdown and terminal renderers
  use `{}` instead of `{:?}`.
- `gamedoc completions <shell>` subcommand — generates shell completion
  scripts for Bash, Zsh, Fish, PowerShell, and Elvish via `clap_complete`.
- `gamedoc man` subcommand — generates a roff man page via `clap_mangen`.
- `--no-color` flag and `NO_COLOR` environment variable support in the
  terminal renderer.
- `--share --format json` and all other `--format` / `--share` combinations
  now work without conflict.
- Coloured terminal output with severity-matched ANSI highlighting.
- `.github/CODEOWNERS` assigning review responsibility for
  architecture-sensitive files.
- `.github/dependabot.yml` for weekly Cargo and GitHub Actions updates.

### Fixed

- `CpuCollector`: filter closure pattern `|&&f|` produced an unsized `str`
  binding; corrected to `|f|`.

### Changed

- JSON field `diagnostics` renamed to `checks` in the `gamedoc.v2` schema.
- Markdown facts table column `System` renamed to `Category`.
- Fact collector execution parallelised with `std::thread::scope`; output
  ordering is deterministic by registry position.
- `ProcessCommandRunner` timeout enforcement replaced with `wait_timeout`
  (no busy-wait).

>>>>>>> 2109919 (v0.2.0)

## [0.1.0] - 2026-07-30

### Added
<<<<<<< HEAD
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
=======

- Read-only system and graphics facts.
- Native, Flatpak, and Snap Steam discovery.
- Repeated Steam libraries, Proton runtimes, and compatibility tools.
- MangoHud and GameMode observations.
- Pure rule evaluation and deterministic problem synthesis.
- Terminal, Markdown, and `gamedoc.v1` JSON reports.
- Privacy-safe finalization and stricter `--share` output.
- Fixture-driven unit tests and full-engine golden scenarios.
- GitHub Actions checks for stable Rust and the Rust 1.74 MSRV.
>>>>>>> 2109919 (v0.2.0)
