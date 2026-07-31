# Contributing to GameDoc

<<<<<<< HEAD
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
=======
Thank you for helping improve Linux gaming diagnostics.

## Before You Start

Read [ARCHITECTURE.md](ARCHITECTURE.md). Its invariants are mandatory:
collectors observe, rules evaluate immutable facts, synthesis creates final
problems, and renderers only format finalized reports.

Open an issue before proposing a schema-breaking change or architectural
exception. Do not add network access, telemetry, shell execution, mutation, or
package-management behavior.

## Development Setup

GameDoc supports Rust 1.74 and newer.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --doc
```

## Adding a Collector

1. Add stable typed IDs in `src/ids.rs`.
2. Read the machine only through `SystemProbe`.
3. Emit facts and sanitized diagnostics, never health or recommendations.
4. Preserve repeated observations rather than overwriting values.
5. Add deterministic `FixtureSystemProbe` tests.
6. Add or extend a full-engine golden scenario under `fixtures/`.

Do not include raw home paths, account IDs, environment values, command stderr,
or launch arguments in facts, provenance, or diagnostics.

## Golden Fixtures

Each scenario contains:

```text
fixtures/<scenario>/commands.json
fixtures/<scenario>/files.json
fixtures/<scenario>/expected.json
fixtures/<scenario>/expected.md
fixtures/<scenario>/expected.txt
```

After an intentional output change, regenerate and review goldens:

```bash
GAMEDOC_UPDATE_GOLDENS=1 cargo test --test golden
cargo test --test golden
```

Never approve generated output without checking ordering, privacy, and schema
compatibility.

## Pull Requests

- Keep changes focused and explain design tradeoffs.
- Add tests for success, absence, unreadable state, and malformed evidence.
- Update public docs and `CHANGELOG.md` when behavior changes.
- Do not include generated build artifacts.
- Ensure all quality gates pass without warnings.
>>>>>>> 2109919 (v0.2.0)
