# GameDoc v0.2.0 - "The Diagnostic Clarity Release"

GameDoc v0.2.0 is a major milestone focusing on structured recommendations, privacy improvements, supply-chain security, and robust documentation. It upgrades the core architecture from strings to structured diagnostic fixes, allowing platform-specific recommendations that are tailored to the host distribution.

## ✨ Highlights

- **Distro-Aware Recommendations:** Actionable fixes now render correctly based on your OS (`apt` for Ubuntu/Debian, `dnf` for Fedora, `pacman` for Arch).
- **Privacy Hardening:** Extensive redactions for the `$USER` variable and local build-host strings ensure reports remain safe to share publicly.
- **Supply-Chain Secure:** Fully verified CI pipeline featuring CycloneDX SBOM generation, OIDC keyless signing with Cosign, and strict dependency audits.
- **New `gamedoc.v2` JSON Schema:** Built with an explicit `schema_url` binding, enabling powerful tooling to automatically validate JSON outputs.

## 🚀 Supported Platforms

GameDoc is currently tested and supported on:
- Arch Linux & Derivatives (SteamOS, Bazzite, EndeavourOS)
- Fedora Workstation & Silverblue
- Ubuntu, Pop!_OS, and Debian
- NixOS (via standard channels)

## 📦 Installation

To install the latest release, you can use Cargo from the [crates.io](https://crates.io/crates/gamedoc) registry:

```bash
cargo install gamedoc
```

*Pre-built binaries and SBOM attestations are available in the Assets section below.*

## 📖 Example Usage

Run a quick summary diagnostic in the terminal:
```bash
gamedoc
```

Share a privacy-scrubbed JSON report for bug tracking:
```bash
gamedoc --share --format json > report.json
```

Generate shell completions (e.g., for bash):
```bash
gamedoc completions bash > ~/.local/share/bash-completion/completions/gamedoc
```

## 🔗 Links
- [Crates.io Package](https://crates.io/crates/gamedoc)
- [Docs.rs Documentation](https://docs.rs/gamedoc)

## 🙌 Acknowledgements
Thank you to all contributors and users who reported diagnostic edge cases and suggested package-manager heuristics!

## 🛣️ Future Roadmap
For the next milestones (v0.3.0 and beyond), we are focusing on:
- **MangoHud Config Key Scanning:** Parsing specific keys (like `fps_limit` and `gpu_stats`) to detect misconfigurations.
- **Enhanced NUMA Support:** Accurately calculating physical core counts via topology IDs for heavily threaded workstations.
- **Native Packaging:** Offering `.deb`, `.rpm`, and Flatpak releases directly.
