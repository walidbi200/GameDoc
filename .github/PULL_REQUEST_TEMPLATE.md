<<<<<<< HEAD
## What does this PR do?
<!-- Describe the purpose of this PR -->

## Motivation and Context
<!-- Why is this change required? What problem does it solve? -->

## How Has This Been Tested?
<!-- Please describe in detail how you tested your changes. -->
<!-- Include details of your testing environment, and the tests you ran to -->
<!-- see how your change affects other areas of the code, etc. -->

- [ ] `cargo test` passes
- [ ] `cargo clippy` is clean
- [ ] Read-only invariants are maintained (no `sh -c`, `std::fs::write`, etc.)

## Screenshots (if appropriate):
<!-- For CLI output changes, please provide a before/after screenshot or codeblock -->
=======
## Summary

Describe the user-visible and architectural changes.

## Architecture

- [ ] I read `ARCHITECTURE.md`.
- [ ] Collectors only observe through `SystemProbe`.
- [ ] Rules consume immutable facts only.
- [ ] Synthesis consumes rule outcomes only.
- [ ] Renderers consume finalized reports only.

## Privacy and Safety

- [ ] No private paths, account IDs, environment values, raw stderr, or secrets enter reports.
- [ ] No writes, network access, shell execution, package operations, or service changes were added.

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features`
- [ ] `cargo test --doc`
- [ ] Goldens were reviewed if output changed.
>>>>>>> 2109919 (v0.2.0)
