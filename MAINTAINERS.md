# Maintainers

GameDoc is maintained by a core team of systems engineers.

## Core Maintainers
- Walidbi200

## Merge Policy
- All PRs must pass the stable quality gate (fmt, clippy, tests).
- All PRs must not introduce external telemetry or mutations.
- The `SystemProbe` trait must remain the only boundary for OS I/O.
- Maintainers are responsible for pushing tags and cutting releases.
