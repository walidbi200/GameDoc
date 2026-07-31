# Security Policy

<<<<<<< HEAD
## Reporting a Vulnerability

GameDoc is a read-only tool designed to be perfectly safe to run on any Linux system without root privileges.
If you believe you have found a security vulnerability (e.g., shell injection through malicious environment variables, unintentional local file overwrites, privilege escalation), please report it to us immediately.

1. **Do not open a public issue.**
2. Send a report via the standard GitHub Security Advisory process under the `Security` tab of this repository.

We will acknowledge receipt of your vulnerability report within 48 hours and strive to send you regular updates about our progress. If you're unsure whether something is a vulnerability, please report it anyway.

## Supported Versions

Currently, only the latest stable version of GameDoc is supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |
=======
## Supported Versions

Until a newer release exists, security fixes target GameDoc 0.1.x.

## Reporting a Vulnerability

Use GitHub private vulnerability reporting when the public repository enables
it. If that is unavailable, contact the repository owner privately. Do not open
a public issue for vulnerabilities involving command execution, path exposure,
privacy leakage, or unsafe report content.

Include the affected version, reproduction steps, impact, and whether the issue
requires unusual local files or environment state. Do not include real secrets
or account identifiers.

## Security Model

GameDoc is local and read-only. It executes only fixed programs with explicit
argument vectors, never a shell. It performs no network requests or telemetry.
Collectors have no write API.

Reports intentionally disclose diagnostic facts such as hardware models and
software versions. Privacy finalization removes home/user paths and control
characters. Raw stderr is never included. `--share` omits diagnostics.

GameDoc does not claim to sandbox external diagnostic tools. It relies on the
locally installed `uname`, `lspci`, `vulkaninfo`, `glxinfo`, and `gamemoded`
executables selected through the process environment.
>>>>>>> 2109919 (v0.2.0)
