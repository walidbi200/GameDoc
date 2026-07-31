# Release Checklist

## Code and Architecture

- [ ] `ARCHITECTURE.md` invariants remain true.
- [ ] Public IDs and `gamedoc.v1` fields are backward-compatible.
- [ ] Collectors remain read-only and rules remain pure.
- [ ] No raw stderr, private paths, account IDs, or environment values appear.

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features`
- [ ] `cargo test --doc`
- [ ] `cargo build --release`
- [ ] Rust 1.74 CI passes.
- [ ] Stable Rust CI passes.
- [ ] Real-machine normal and `--share` reports are reviewed.

## Documentation

- [ ] `README.md`, `CHANGELOG.md`, and examples match behavior.
- [ ] Version and release date are correct.
- [ ] Security reporting is enabled and documented.

## Publication

- [ ] Tag `v0.1.0` from the reviewed commit.
- [ ] Build release artifacts in a clean environment.
- [ ] Publish checksums with artifacts.
- [ ] Verify the downloaded artifact before announcing the release.
