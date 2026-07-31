# Release Notes v0.2.0

## Features
- Structured `Recommendation` type with platform-specific instructions.
- Compile-time severity registration.
- Privacy enhancements: `$USER` and build-host marker redaction.
- Distro-aware recommendations for package manager commands.
- Software Supply Chain CI (`cargo-audit`, `cargo-deny`, `cosign`, CycloneDX).

## Changes
- `diagnostics` renamed to `checks` in JSON schema.
- Schema explicitly references `gamedoc.v2`.
