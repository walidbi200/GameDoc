# Frequently Asked Questions

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
