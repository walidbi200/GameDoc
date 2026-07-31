# Adding a New Collector to GameDoc

GameDoc uses a data-driven approach to system diagnostics. If you want to add a new check, you must write a *Collector*.

## 1. Define the Facts
First, determine what raw data your collector will observe. Add your new fact identifiers to `src/ids.rs` in the `fact` module.

## 2. Implement `Collector`
Create a new file in `src/collect/` (e.g., `src/collect/my_tool.rs`).
Implement the `Collector` trait:
```rust
use crate::collect::{Collection, Collector};
use crate::probe::SystemProbe;

pub struct MyToolCollector;

impl Collector for MyToolCollector {
    fn id(&self) -> &'static str {
        crate::ids::collector::MY_TOOL
    }

    fn collect(&self, system: &dyn SystemProbe) -> Collection {
        let mut collection = Collection::default();
        // Use system.command() or system.read() to gather facts
        collection
    }
}
```

## 3. Register the Collector
Add your collector to the `COLLECTORS` array in `src/collect/mod.rs`.

## 4. Add Rules
Data is useless without interpretation. Create rules in `src/rules/` that evaluate the facts your collector produces and yield `RuleOutcome`s.

## 5. Add Recommendations
If your rule can yield a problem, define the recommendation in `src/synthesize.rs` with actionable, platform-specific instructions.
