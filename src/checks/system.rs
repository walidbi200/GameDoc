//! System-level diagnostics collector.
//!
//! Checks emitted by this collector:
//!
//! | ID | Source | Notes |
//! |----|--------|-------|
//! | `system.kernel_version` | `/proc/version` | First line only |
//! | `system.architecture` | `uname -m` | Falls back to compile-time constant |
//! | `system.session_type` | `XDG_SESSION_TYPE` env var | `wayland` / `x11` / unknown |
//! | `system.hostname` | `/etc/hostname` | Value is never exposed |
//! | `system.memory` | `/proc/meminfo` | `MemTotal` + `MemAvailable` |

use std::path::Path;

use crate::collect::context::CollectorContext;
use crate::collect::{CollectionResult, Collector};
use crate::ids;
use crate::model::{Category, Check, Confidence, Evidence, EvidenceSource, Fact, Status, Trace};

/// Collector for basic Linux system properties.
pub struct SystemCollector;

impl Collector for SystemCollector {
    fn id(&self) -> &'static str {
        "system"
    }

    fn collect(&self, ctx: &CollectorContext) -> CollectionResult {
        let mut res = CollectionResult::new();
        check_kernel_version(ctx, &mut res);
        check_architecture(ctx, &mut res);
        check_session_type(&mut res);
        check_hostname(ctx, &mut res);
        check_memory(ctx, &mut res);
        res
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn push_fact(res: &mut CollectionResult, id: &str, value: impl Into<String>, evidence: Evidence) {
    res.facts.push(Fact {
        id: id.to_owned(),
        category: Category::System,
        value: value.into(),
        evidence,
    });
}

fn push_check(
    res: &mut CollectionResult,
    id: &str,
    status: Status,
    confidence: Confidence,
    summary: impl Into<String>,
    details: Vec<String>,
    evidence: Evidence,
) {
    res.checks.push(Check {
        id: id.to_owned(),
        category: Category::System,
        status,
        confidence,
        summary: summary.into(),
        details,
        recommendation: None,
        evidence,
    });
}

// ---------------------------------------------------------------------------
// Individual check functions
// ---------------------------------------------------------------------------

fn check_kernel_version(ctx: &CollectorContext, res: &mut CollectionResult) {
    let evidence = Evidence::new(EvidenceSource::FileSystem, "/proc/version");
    if let Ok(content) = ctx.fs.read_to_string(Path::new("/proc/version")) {
        let line = content.lines().next().unwrap_or("").trim();
        if line.is_empty() {
            push_check(
                res,
                ids::system::KERNEL_VERSION,
                Status::Warning,
                Confidence::Unknown,
                "Kernel version string is empty",
                vec!["/proc/version exists but contains no content".to_owned()],
                evidence,
            );
        } else {
            push_fact(res, ids::system::KERNEL_VERSION, line, evidence);
        }
    } else {
        push_check(
            res,
            ids::system::KERNEL_VERSION,
            Status::Warning,
            Confidence::Unknown,
            "Kernel version could not be read",
            vec!["/proc/version was not accessible".to_owned()],
            evidence,
        );
    }
}

fn check_architecture(ctx: &CollectorContext, res: &mut CollectionResult) {
    match ctx.command.run("uname", &["-m"]) {
        Ok(output) => {
            let arch = output.stdout.trim();
            if arch.is_empty() {
                fallback_architecture(res);
            } else {
                let evidence =
                    Evidence::new(EvidenceSource::Command, "uname -m").with_trace(Trace {
                        exit_code: output.exit_code,
                        duration_ms: output.duration_ms,
                    });
                push_fact(res, ids::system::ARCHITECTURE, arch, evidence);
            }
        }
        Err(_) => fallback_architecture(res),
    }
}

/// Use the compile-time architecture constant when `uname` is unavailable.
fn fallback_architecture(res: &mut CollectionResult) {
    push_fact(
        res,
        ids::system::ARCHITECTURE,
        std::env::consts::ARCH,
        Evidence::new(EvidenceSource::FileSystem, "compile_constant"),
    );
    push_check(
        res,
        ids::system::ARCHITECTURE,
        Status::Warning,
        Confidence::Detected,
        "Architecture detected via compile-time constant",
        vec!["uname -m was unavailable or returned empty output".to_owned()],
        Evidence::new(EvidenceSource::Command, "uname -m"),
    );
}

/// Classify the display session type from the value of `XDG_SESSION_TYPE`.
pub(crate) fn classify_session_type(value: Option<&str>, res: &mut CollectionResult) {
    let evidence = Evidence::new(EvidenceSource::FileSystem, "env:XDG_SESSION_TYPE");
    match value {
        Some("wayland") => push_fact(res, ids::system::SESSION_TYPE, "Wayland", evidence),
        Some("x11" | "X11" | "xcb") => push_fact(res, ids::system::SESSION_TYPE, "X11", evidence),
        Some(other) => {
            push_fact(res, ids::system::SESSION_TYPE, other, evidence.clone());
            push_check(
                res,
                ids::system::SESSION_TYPE,
                Status::Warning,
                Confidence::Detected,
                "Unrecognised session type",
                vec!["Display server assumed available".to_owned()],
                evidence,
            );
        }
        None => push_check(
            res,
            ids::system::SESSION_TYPE,
            Status::Warning,
            Confidence::Unknown,
            "Unknown (XDG_SESSION_TYPE not set)",
            Vec::new(),
            evidence,
        ),
    }
}

fn check_session_type(res: &mut CollectionResult) {
    let value = std::env::var("XDG_SESSION_TYPE").ok();
    classify_session_type(value.as_deref(), res);
}

fn check_hostname(ctx: &CollectorContext, res: &mut CollectionResult) {
    let evidence = Evidence::new(EvidenceSource::FileSystem, "/etc/hostname");
    match ctx.fs.read_to_string(Path::new("/etc/hostname")) {
        Ok(content) if !content.trim().is_empty() => {
            push_fact(res, ids::system::HOSTNAME, "Hostname detected", evidence);
        }
        Ok(_) => {
            push_check(
                res,
                ids::system::HOSTNAME,
                Status::Warning,
                Confidence::Unknown,
                "Hostname file is empty",
                vec!["/etc/hostname exists but contains no hostname".to_owned()],
                evidence,
            );
        }
        Err(_) => {
            push_check(
                res,
                ids::system::HOSTNAME,
                Status::Warning,
                Confidence::Unknown,
                "Hostname could not be read",
                Vec::new(),
                evidence,
            );
        }
    }
}

fn check_memory(ctx: &CollectorContext, res: &mut CollectionResult) {
    let evidence = Evidence::new(EvidenceSource::FileSystem, "/proc/meminfo");
    if let Ok(content) = ctx.fs.read_to_string(Path::new("/proc/meminfo")) {
        if let Some((total_kb, available_kb)) = parse_meminfo(&content) {
            push_fact(
                res,
                ids::system::MEMORY,
                format!(
                    "{} total, {} available",
                    format_gib(total_kb),
                    format_gib(available_kb)
                ),
                evidence,
            );
        } else {
            push_check(
                res,
                ids::system::MEMORY,
                Status::Warning,
                Confidence::Unknown,
                "Memory information could not be parsed",
                vec!["MemTotal or MemAvailable was missing from /proc/meminfo".to_owned()],
                evidence,
            );
        }
    } else {
        push_check(
            res,
            ids::system::MEMORY,
            Status::Warning,
            Confidence::Unknown,
            "Memory information could not be read",
            Vec::new(),
            evidence,
        );
    }
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

fn parse_meminfo(content: &str) -> Option<(u64, u64)> {
    let mut total: Option<u64> = None;
    let mut available: Option<u64> = None;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total = parse_kb(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available = parse_kb(rest);
        }
        if total.is_some() && available.is_some() {
            break;
        }
    }

    Some((total?, available?))
}

fn parse_kb(s: &str) -> Option<u64> {
    s.split_whitespace().next()?.parse().ok()
}

#[allow(clippy::cast_precision_loss)]
fn format_gib(kb: u64) -> String {
    let gib = kb as f64 / (1024.0 * 1024.0);
    format!("{gib:.1} GiB")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::{classify_session_type, format_gib, SystemCollector};
    use crate::collect::command::{CommandOutput, CommandRunner};
    use crate::collect::context::CollectorContext;
    use crate::collect::filesystem::StubFilesystem;
    use crate::collect::{CollectionResult, Collector};
    use crate::errors::CollectError;
    use crate::ids;
    use crate::model::{Confidence, Status};

    const PROC_VERSION: &str = include_str!("../../tests/fixtures/proc/version");
    const PROC_MEMINFO: &str = include_str!("../../tests/fixtures/proc/meminfo");
    const ETC_HOSTNAME: &str = include_str!("../../tests/fixtures/etc/hostname");

    struct StubCommandRunner(HashMap<String, String>);

    impl CommandRunner for StubCommandRunner {
        fn run(&self, program: &str, _args: &[&str]) -> Result<CommandOutput, CollectError> {
            self.0.get(program).map_or_else(
                || {
                    Err(CollectError::CommandNotFound {
                        program: program.to_owned(),
                    })
                },
                |stdout| {
                    Ok(CommandOutput {
                        stdout: stdout.clone(),
                        stderr: String::new(),
                        exit_code: Some(0),
                        duration_ms: 10,
                    })
                },
            )
        }
    }

    fn ctx(fs: StubFilesystem) -> CollectorContext {
        CollectorContext::new(
            Arc::new(crate::collect::command::ProcessCommandRunner::new()),
            Arc::new(fs),
        )
    }

    fn ctx_with_cmd(cmd: StubCommandRunner, fs: StubFilesystem) -> CollectorContext {
        CollectorContext::new(Arc::new(cmd), Arc::new(fs))
    }

    #[test]
    fn kernel_version_detected_from_fixture() {
        let fs = StubFilesystem::new().with_file("/proc/version", PROC_VERSION);
        let res = SystemCollector.collect(&ctx(fs));
        let k = res
            .facts
            .iter()
            .find(|c| c.id == ids::system::KERNEL_VERSION)
            .unwrap();

        assert!(
            k.value.starts_with("Linux version 6.9.0"),
            "expected kernel version, got: {}",
            k.value
        );
    }

    #[test]
    fn kernel_version_unknown_when_file_missing() {
        let res = SystemCollector.collect(&ctx(StubFilesystem::new()));
        let k = res
            .checks
            .iter()
            .find(|c| c.id == ids::system::KERNEL_VERSION)
            .unwrap();

        assert_eq!(k.status, Status::Warning);
        assert_eq!(k.confidence, Confidence::Unknown);
    }

    #[test]
    fn architecture_detected_via_uname() {
        let mut cmds = HashMap::new();
        cmds.insert("uname".to_owned(), "x86_64\n".to_owned());

        let res = SystemCollector.collect(&ctx_with_cmd(
            StubCommandRunner(cmds),
            StubFilesystem::new(),
        ));
        let a = res
            .facts
            .iter()
            .find(|c| c.id == ids::system::ARCHITECTURE)
            .unwrap();
        assert_eq!(a.value, "x86_64");
    }

    #[test]
    fn architecture_falls_back_to_compile_constant_when_uname_missing() {
        let cmds = HashMap::new();
        let res = SystemCollector.collect(&ctx_with_cmd(
            StubCommandRunner(cmds),
            StubFilesystem::new(),
        ));
        let a_fact = res
            .facts
            .iter()
            .find(|c| c.id == ids::system::ARCHITECTURE)
            .unwrap();
        let a_check = res
            .checks
            .iter()
            .find(|c| c.id == ids::system::ARCHITECTURE)
            .unwrap();

        assert!(!a_fact.value.is_empty(), "fallback arch must not be empty");
        assert_eq!(a_check.status, Status::Warning);
    }

    #[test]
    fn session_type_wayland() {
        let mut res = CollectionResult::new();
        classify_session_type(Some("wayland"), &mut res);
        let f = res
            .facts
            .iter()
            .find(|c| c.id == ids::system::SESSION_TYPE)
            .unwrap();
        assert_eq!(f.value, "Wayland");
    }

    #[test]
    fn session_type_unknown_when_unset() {
        let mut res = CollectionResult::new();
        classify_session_type(None, &mut res);
        let c = res
            .checks
            .iter()
            .find(|c| c.id == ids::system::SESSION_TYPE)
            .unwrap();
        assert_eq!(c.status, Status::Warning);
        assert_eq!(c.confidence, Confidence::Unknown);
    }

    #[test]
    fn hostname_detected_from_fixture() {
        let fs = StubFilesystem::new().with_file("/etc/hostname", ETC_HOSTNAME);
        let res = SystemCollector.collect(&ctx(fs));
        let h = res
            .facts
            .iter()
            .find(|c| c.id == ids::system::HOSTNAME)
            .unwrap();
        assert_eq!(h.value, "Hostname detected");
    }

    #[test]
    fn memory_parsed_from_fixture() {
        let fs = StubFilesystem::new().with_file("/proc/meminfo", PROC_MEMINFO);
        let res = SystemCollector.collect(&ctx(fs));
        let m = res
            .facts
            .iter()
            .find(|c| c.id == ids::system::MEMORY)
            .unwrap();

        assert!(m.value.contains("31.2 GiB total"));
        assert!(m.value.contains("23.4 GiB available"));
    }

    #[test]
    fn format_gib_rounds_to_one_decimal() {
        assert_eq!(format_gib(1_048_576), "1.0 GiB");
        assert_eq!(format_gib(32_765_192), "31.2 GiB");
    }
}
