# Benchmarks

Execution time was measured across 5 runs of the optimized release build (`cargo build --release`) on the host system.

## Results
- Run 1: 0.647s
- Run 2: 2.687s
- Run 3: 0.068s
- Run 4: 2.709s
- Run 5: 0.073s

**Median Execution Time:** ~0.647 seconds

_Note: Slower runs (~2.7s) reflect command runner timeouts (e.g., missing daemons or unreadable configurations gracefully timing out at the 2.5s limit) in parallel execution._
