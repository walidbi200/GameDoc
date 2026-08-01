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
    }
}
