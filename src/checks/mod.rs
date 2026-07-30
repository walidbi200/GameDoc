//! Diagnostic check implementations.
//!
//! Each sub-module contains one [`crate::collect::Collector`] implementation.
//! To register a new collector, add it to [`collect`].
//!
//! Collectors are intentionally not public — they are accessed through
//! [`collect`], which is the single place the application builds the full
//! list of checks to run.

pub mod gpu;
pub mod steam;
pub mod system;

use crate::collect::{context::CollectorContext, CollectionResult, Collector};

use gpu::GpuCollector;
use steam::SteamCollector;
use system::SystemCollector;

/// Run every registered collector and return all resulting checks.
///
/// This is the only function `main` needs to call.  New collectors are
/// registered here, not in `main`.
#[must_use]
pub fn collect(ctx: &CollectorContext) -> CollectionResult {
    let collectors: Vec<Box<dyn Collector>> = vec![
        Box::new(SystemCollector),
        Box::new(GpuCollector),
        Box::new(SteamCollector),
    ];

    let mut final_res = CollectionResult::new();
    for c in collectors {
        let res = c.collect(ctx);
        final_res.facts.extend(res.facts);
        final_res.checks.extend(res.checks);
        final_res.problems.extend(res.problems);
    }
    final_res
}

#[cfg(test)]
mod tests {
    use super::collect;
    use crate::collect::context::CollectorContext;

    #[test]
    fn collect_returns_nonempty_results_on_real_system() {
        let ctx = CollectorContext::real();
        let res = collect(&ctx);
        assert!(
            !res.facts.is_empty() || !res.checks.is_empty(),
            "collect() must return at least one fact or check on a real Linux system"
        );
    }
}
