# Frequently Asked Questions

<<<<<<< HEAD
### What is GameDoc?
GameDoc is a deterministic, read-only diagnostics tool for Linux gaming. It quickly scans your hardware and software environment and produces a shareable report about your system state.

### Does it require root / sudo?
**No.** GameDoc is designed to operate strictly in user space without requiring `sudo` or root privileges. It will never ask for your password.

### Will it modify my system?
**Never.** GameDoc guarantees a 100% read-only operation. We enforce this through rigorous automated tests (using `unshare` to restrict file access to read-only during CI) and strict code linting to ban file writing methods.

### Can it fix problems for me?
GameDoc intentionally avoids automatic fixes or "tweaks". Its sole purpose is to report the current state of your machine so you can share it with others or fix it manually based on the provided recommendations.

### Is GameDoc tracking my data?
**No.** GameDoc does not include any telemetry, analytics, or network-calling code. It operates entirely offline.

### What if I don't want to share my username online?
You can use the `--share` flag (e.g., `gamedoc --share`). This will automatically sanitize the output by replacing your `/home/username` paths with a generic `~` placeholder.
=======
## Does GameDoc change my system?

No. It reads local state and executes fixed diagnostic commands. It does not
write configuration, install software, start services, or launch games.

## Is the report safe to post?

Default reports are privacy-finalized and exclude usernames, home paths,
hostnames, account IDs, environment values, launch arguments, raw stderr, and
control characters. `--share` also removes incomplete-observation diagnostics.
Always review any diagnostic report before posting it publicly.

## Why does GameDoc say an observation is incomplete?

Missing tools, unreadable paths, unavailable display sessions, and permission
errors are evidence limitations. GameDoc records them rather than guessing.

## Why is missing software not always a problem?

Many tools are optional. A missing probe also does not prove the underlying
capability is absent. Rules report problems only when sufficient evidence is
available.

## Does GameDoc launch Steam or inspect my account?

No. It reads supported local Steam directories. It does not launch Steam and
does not report Steam account IDs.

## Why are Steam libraries numbered instead of showing paths?

Library ordinals preserve repeated observations without exposing usernames,
mount names, or private directory structures.

## Does GameDoc support Minecraft?

GameDoc does not currently include game-specific collectors. Minecraft is a
useful real-world validation workload, but GameDoc reports the surrounding
Linux gaming environment rather than inspecting individual games.
>>>>>>> 2109919 (v0.2.0)
