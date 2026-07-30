# Contributing to GameDoc

Contributions are welcome! Please ensure that any additions respect the core philosophy of GameDoc:
* **Read-only**: Never mutate the user's system.
* **Deterministic**: Try to avoid unstable timing or sorting.
* **No Telemetry/Network**: Never contact the internet.
* **No Shell Scripts**: Do not use `sh -c` or arbitrary bash pipelines. Use the `CommandRunner` abstraction.

## Development Setup

1. Install Rust (`rustup default stable`).
2. Clone the repository.
3. Run `cargo test` to ensure everything works locally.

## Running Tests

GameDoc relies heavily on mocked filesystems and command runners to test hardware detection logic safely.

```bash
# Run the test suite
cargo test --all-targets --all-features

# Verify formatting and linting
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Read-Only Invariants

GameDoc uses a strict `clippy.toml` to ban mutating standard library methods. If you legitimately need an exception (which should be extremely rare), it must be discussed in an issue first.

You can verify the read-only guarantees locally using the test script:
```bash
cargo build --release
bash tests/readonly_check.sh
```

## Workaround for root-owned `target/` directory
In some CI and Docker environments, the `target/` directory may become owned by `root`. If you encounter permission issues when building locally, please be aware that this is a known environment artifact and not a flaw in the build process itself. You can safely `sudo chown -R $USER:$USER target/` or use an alternative target directory via the `CARGO_TARGET_DIR` environment variable.
