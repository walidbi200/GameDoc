<<<<<<< HEAD
//! Diagnostic data collection.
//!
//! This module defines the [`Collector`] trait and re-exports the supporting
//! types that every collector implementation depends on.

pub mod command;
pub mod context;
pub mod filesystem;

use crate::model::{Check, Fact, Problem};

use context::CollectorContext;

/// The structured result of a single collector.
#[derive(Clone, Debug, Default)]
pub struct CollectionResult {
    pub facts: Vec<Fact>,
    pub checks: Vec<Check>,
    pub problems: Vec<Problem>,
}

impl CollectionResult {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_fact(mut self, fact: Fact) -> Self {
        self.facts.push(fact);
        self
    }

    #[must_use]
    pub fn with_check(mut self, check: Check) -> Self {
        self.checks.push(check);
        self
    }

    #[must_use]
    pub fn with_problem(mut self, problem: Problem) -> Self {
        self.problems.push(problem);
        self
    }
}

/// A self-contained diagnostic check (or group of checks) for a single
/// subsystem.
///
/// Implementations are encouraged to:
///
/// - Group closely related checks into a single collector.
/// - Return `Confidence::Unknown` when evidence is ambiguous.
/// - Return `Confidence::Error` when an operation fails unexpectedly.
/// - Never panic.
/// - Never modify the system.
pub trait Collector: Send + Sync {
    /// A stable, human-readable identifier for this collector used in logging
    /// and diagnostics.
    fn id(&self) -> &'static str;

    /// Run all checks for this collector and return the results.
    ///
    /// This method must not propagate errors.  Any failure during collection
    /// must be represented as a [`Check`] with an appropriate
    /// [`crate::model::Confidence`].
    fn collect(&self, ctx: &CollectorContext) -> CollectionResult;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::context::CollectorContext;
    use super::{CollectionResult, Collector};
    use crate::collect::filesystem::StubFilesystem;
    use crate::model::{Category, Check, Confidence, Evidence, EvidenceSource, Status};

    /// Minimal collector used to test the trait plumbing.
    struct AlwaysHealthy;

    impl Collector for AlwaysHealthy {
        fn id(&self) -> &'static str {
            "test.always_healthy"
        }

        fn collect(&self, _ctx: &CollectorContext) -> CollectionResult {
            let mut res = CollectionResult::new();
            res.checks.push(Check {
                id: "test.always_healthy".to_owned(),
                category: Category::System,
                status: Status::Healthy,
                confidence: Confidence::Detected,
                summary: "Always healthy".to_owned(),
                details: Vec::new(),
                recommendation: None,
                evidence: Evidence::new(EvidenceSource::FileSystem, "test"),
            });
            res
        }
    }

    fn make_ctx() -> CollectorContext {
        CollectorContext::new(
            Arc::new(crate::collect::command::ProcessCommandRunner::new()),
            Arc::new(StubFilesystem::new()),
        )
    }

    #[test]
    fn collector_id_is_stable() {
        let c = AlwaysHealthy;
        assert_eq!(c.id(), "test.always_healthy");
    }

    #[test]
    fn collector_returns_checks() {
        let c = AlwaysHealthy;
        let ctx = make_ctx();
        let res = c.collect(&ctx);
        assert_eq!(res.checks.len(), 1);
        assert_eq!(res.checks[0].status, Status::Healthy);
        assert_eq!(res.checks[0].confidence, Confidence::Detected);
    }

    #[test]
    fn collector_checks_can_be_empty() {
        struct NeverCollects;

        impl Collector for NeverCollects {
            fn id(&self) -> &'static str {
                "test.never"
            }

            fn collect(&self, _ctx: &CollectorContext) -> CollectionResult {
                CollectionResult::new()
            }
        }

        let ctx = make_ctx();
        let res = NeverCollects.collect(&ctx);
        assert!(res.facts.is_empty());
        assert!(res.checks.is_empty());
        assert!(res.problems.is_empty());
=======
//! Read-only collectors that emit observations without evaluating health.

pub mod command;
mod cpu;
mod distro;
mod gamemode;
mod graphics;
mod mangohud;
mod steam;
mod system;

use serde::Serialize;

use crate::facts::{Fact, FactBuilder, Facts};
use crate::ids::CollectorId;
use crate::probe::SystemProbe;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct CollectionResult {
    pub facts: Vec<Fact>,
    pub diagnostics: Vec<CollectorDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CollectorDiagnostic {
    pub collector: CollectorId,
    pub operation: String,
    pub message: String,
}

pub trait Collector: Send + Sync {
    fn id(&self) -> CollectorId;
    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult;
}

/// Collect facts and diagnostics from all registered collectors in parallel.
///
/// # Panics
///
/// Panics if any collector thread panics. This indicates a programming defect
/// in a collector rather than an expected host condition.
#[must_use]
pub fn collect_all(system: &dyn SystemProbe) -> (Facts, Vec<CollectorDiagnostic>) {
    let collectors: [&dyn Collector; 7] = [
        &system::SystemCollector,
        &cpu::CpuCollector,
        &distro::DistroCollector,
        &graphics::GraphicsCollector,
        &steam::SteamCollector,
        &mangohud::MangoHudCollector,
        &gamemode::GameModeCollector,
    ];

    let mut results = Vec::with_capacity(collectors.len());

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(collectors.len());
        for collector in collectors {
            handles.push(scope.spawn(|| collector.collect(system)));
        }
        for handle in handles {
            results.push(handle.join().expect("collector panicked"));
        }
    });

    let mut builder = FactBuilder::new();
    let mut diagnostics = Vec::new();
    for result in results {
        builder.append(result.facts);
        diagnostics.extend(result.diagnostics);
    }

    (builder.build(), diagnostics)
}

pub(crate) fn diagnostic(
    collector: CollectorId,
    operation: impl Into<String>,
    error: impl std::fmt::Display,
) -> CollectorDiagnostic {
    CollectorDiagnostic {
        collector,
        operation: operation.into(),
        message: error.to_string(),
>>>>>>> 2109919 (v0.2.0)
    }
}
