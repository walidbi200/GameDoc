<<<<<<< HEAD
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Report {
    pub schema: String,
    pub generator: String,
    pub version: String,
    pub facts: Vec<Fact>,
    #[serde(rename = "diagnostics")]
    pub checks: Vec<Check>,
    pub problems: Vec<Problem>,
=======
//! Finalized, renderer-independent public report model.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::collect::CollectorDiagnostic;
use crate::facts::{Category, Fact, Facts};
use crate::ids::ProblemId;

pub const SCHEMA: &str = "gamedoc.v2";
pub const GAMEDOC_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Report {
    pub schema: String,
    pub gamedoc_version: String,
    pub facts: Vec<Fact>,
    pub problems: Vec<Problem>,
    pub checks: Vec<CollectorDiagnostic>,
>>>>>>> 2109919 (v0.2.0)
}

impl Report {
    #[must_use]
<<<<<<< HEAD
    pub fn new(
        gamedoc_version: impl Into<String>,
        facts: Vec<Fact>,
        checks: Vec<Check>,
        problems: Vec<Problem>,
    ) -> Self {
        Self {
            schema: "gamedoc.v1".to_owned(),
            generator: "gamedoc".to_owned(),
            version: gamedoc_version.into(),
            facts,
            checks,
            problems,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Fact {
    pub id: String,
    pub category: Category,
    pub value: String,
    pub evidence: Evidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Check {
    pub id: String,
    pub category: Category,
    pub status: Status,
    pub confidence: Confidence,
    pub summary: String,
    pub details: Vec<String>,
    pub recommendation: Option<String>,
    pub evidence: Evidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Problem {
    pub id: String,
    pub category: Category,
    pub severity: Status,
    pub description: String,
    pub recommendation: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    System,
    Gpu,
    Steam,
    MangoHud,
    GameMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Healthy,
    Warning,
    Problem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Detected,
    NotDetected,
    Unknown,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Evidence {
    pub source: EvidenceSource,
    pub label: String,
    pub trace: Option<Trace>,
}

impl Evidence {
    #[must_use]
    pub fn new(source: EvidenceSource, label: impl Into<String>) -> Self {
        Self {
            source,
            label: label.into(),
            trace: None,
        }
    }

    #[must_use]
    pub fn with_trace(mut self, trace: Trace) -> Self {
        self.trace = Some(trace);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    FileSystem,
    Path,
    Command,
    PackageManager,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
=======
    pub fn new(facts: Facts, problems: Vec<Problem>, checks: Vec<CollectorDiagnostic>) -> Self {
        Self {
            schema: SCHEMA.to_owned(),
            gamedoc_version: GAMEDOC_VERSION.to_owned(),
            facts: facts.into_vec(),
            problems,
            checks,
        }
    }

    /// Returns a privacy-scrubbed version of the report suitable for public sharing.
    ///
    /// This method enforces a strict privacy contract:
    /// - Replaces all occurrences of `/home/<username>` with `$HOME`.
    /// - Replaces the current `$USER` environment variable value with `$USER`.
    /// - Redacts build-host strings (e.g. `user@host` in kernel version) to `redacted@redacted`.
    /// - Replaces control characters with spaces.
    /// - Optionally removes all incomplete collector diagnostics.
    #[must_use]
    pub fn public_safe(mut self, omit_diagnostics: bool) -> Self {
        for fact in &mut self.facts {
            sanitize_value(&mut fact.value);
        }
        for problem in &mut self.problems {
            problem.summary = sanitize_text(&problem.summary);
            problem.details = problem
                .details
                .iter()
                .map(|detail| sanitize_text(detail))
                .collect();
            if let Some(rec) = &mut problem.recommendation {
                rec.text = sanitize_text(&rec.text);
                for val in rec.platform_specific.values_mut() {
                    *val = sanitize_text(val);
                }
            }
        }
        for diagnostic in &mut self.checks {
            diagnostic.operation = sanitize_text(&diagnostic.operation);
            diagnostic.message = sanitize_text(&diagnostic.message);
        }
        if omit_diagnostics {
            self.checks.clear();
        }
        self
    }

    #[must_use]
    pub fn distro_family(&self) -> Option<&str> {
        self.facts
            .iter()
            .find(|f| f.id == crate::ids::fact::DISTRO_FAMILY)
            .and_then(|f| f.value.as_str())
    }

    #[must_use]
    pub fn gaming_score(&self) -> u32 {
        let mut score: i32 = 100;
        for problem in &self.problems {
            let delta = match problem.severity {
                Severity::Critical => 25,
                Severity::High => 10,
                Severity::Medium => 5,
                Severity::Low => 2,
                Severity::Informational => 0,
            };
            score -= delta;
        }
        if let Ok(checks_len) = i32::try_from(self.checks.len()) {
            score -= checks_len;
        }
        if score < 0 {
            0
        } else {
            score.unsigned_abs()
        }
    }

    #[must_use]
    pub fn gaming_status(&self) -> &'static str {
        match self.gaming_score() {
            95..=100 => "Excellent Linux Gaming Environment",
            80..=94 => "Ready for Linux Gaming",
            60..=79 => "Gaming Ready — Minor Issues",
            40..=59 => "Needs Attention",
            _ => "Gaming Environment Has Critical Issues",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    /// Human-readable summary, 1-2 sentences, no embedded newlines.
    pub text: String,
    /// Platform-specific command or instruction, keyed by `distro.family`
    /// value (e.g. "fedora", "arch", "debian", "ubuntu", "nixos",
    /// "steamos", "bazzite", "universal"). Empty map = use `text` only.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub platform_specific: BTreeMap<String, String>,
    /// 1 (trivial) to 5 (rebuild kernel). Use `u8`.
    pub difficulty: u8,
    /// Estimated minutes for an experienced user. Use `u16`. 0 if N/A.
    pub est_minutes: u16,
    /// Whether the fix requires root or sudo.
    pub requires_root: bool,
    /// Whether the fix requires a reboot.
    pub requires_reboot: bool,
    /// Optional upstream docs URL (e.g. Arch wiki, Fedora docs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

impl Recommendation {
    #[must_use]
    pub fn for_distro(&self, distro_family: Option<&str>) -> &str {
        if let Some(family) = distro_family {
            if let Some(specific) = self.platform_specific.get(family) {
                return specific;
            }
        }
        if let Some(universal) = self.platform_specific.get("universal") {
            return universal;
        }
        &self.text
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Problem {
    pub id: ProblemId,
    pub category: Category,
    pub severity: Severity,
    pub summary: String,
    pub details: Vec<String>,
    pub recommendation: Option<Recommendation>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Informational => "informational",
        };
        f.write_str(label)
    }
}

fn sanitize_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => *text = sanitize_text(text),
        serde_json::Value::Array(items) => {
            for item in items {
                sanitize_value(item);
            }
        }
        serde_json::Value::Object(fields) => {
            for item in fields.values_mut() {
                sanitize_value(item);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

fn sanitize_text(text: &str) -> String {
    let normalized: String = text
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();
    let mut s = redact_home_path(&normalized);
    s = redact_build_host(&s);
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() && user != "root" {
            s = s.replace(&user, "$USER");
        }
    }
    s
}

fn redact_build_host(text: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@' {
            word.push(c);
        } else {
            if word.contains('@')
                && word.len() > 1
                && !word.starts_with('@')
                && !word.ends_with('@')
            {
                out.push_str("redacted@redacted");
            } else {
                out.push_str(&word);
            }
            word.clear();
            out.push(c);
        }
    }
    if word.contains('@') && word.len() > 1 && !word.starts_with('@') && !word.ends_with('@') {
        out.push_str("redacted@redacted");
    } else {
        out.push_str(&word);
    }
    out
}

fn redact_home_path(text: &str) -> String {
    let mut output = text.to_owned();
    while let Some(start) = output.find("/home/") {
        let username_start = start + "/home/".len();
        let username_end = output[username_start..]
            .find('/')
            .map_or(output.len(), |offset| username_start + offset);
        if username_end == username_start {
            break;
        }
        output.replace_range(start..username_end, "$HOME");
    }
    output = output.replace("/root/", "$HOME/");
    if output == "/root" {
        "$HOME".to_owned()
    } else {
        output
    }
>>>>>>> 2109919 (v0.2.0)
}

#[cfg(test)]
mod tests {
<<<<<<< HEAD
    use serde_json::json;

    use super::{Category, Check, Confidence, Evidence, EvidenceSource, Fact, Report, Status};

    #[test]
    fn report_uses_current_schema_version() {
        let report = Report::new("0.1.0", Vec::new(), Vec::new(), Vec::new());

        assert_eq!(report.schema, "gamedoc.v1");
        assert_eq!(report.generator, "gamedoc");
        assert_eq!(report.version, "0.1.0");
        assert!(report.facts.is_empty());
        assert!(report.checks.is_empty());
        assert!(report.problems.is_empty());
    }

    #[test]
    fn report_json_shape_is_stable() {
        let report = Report::new(
            "0.1.0",
            vec![Fact {
                id: "system.kernel_version".to_owned(),
                category: Category::System,
                value: "6.9.0".to_owned(),
                evidence: Evidence::new(EvidenceSource::FileSystem, "/proc/version"),
            }],
            vec![Check {
                id: "system.example".to_owned(),
                category: Category::System,
                status: Status::Warning,
                confidence: Confidence::Unknown,
                summary: "Example check".to_owned(),
                details: vec!["No local evidence was available".to_owned()],
                recommendation: None,
                evidence: Evidence::new(EvidenceSource::FileSystem, "missing"),
            }],
            Vec::new(),
        );

        let value = serde_json::to_value(report).expect("report should serialize");

        assert_eq!(
            value,
            json!({
                "schema": "gamedoc.v1",
                "generator": "gamedoc",
                "version": "0.1.0",
                "facts": [{
                    "id": "system.kernel_version",
                    "category": "system",
                    "value": "6.9.0",
                    "evidence": {
                        "source": "file_system",
                        "label": "/proc/version",
                        "trace": null
                    }
                }],
                "diagnostics": [{
                    "id": "system.example",
                    "category": "system",
                    "status": "warning",
                    "confidence": "unknown",
                    "summary": "Example check",
                    "details": ["No local evidence was available"],
                    "recommendation": null,
                    "evidence": {
                        "source": "file_system",
                        "label": "missing",
                        "trace": null
                    }
                }],
                "problems": []
            })
        );
    }

    #[test]
    fn report_round_trips_through_json() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![Check {
                id: "gpu.example".to_owned(),
                category: Category::Gpu,
                status: Status::Healthy,
                confidence: Confidence::Detected,
                summary: "Example GPU check".to_owned(),
                details: Vec::new(),
                recommendation: Some("No action needed".to_owned()),
                evidence: Evidence::new(EvidenceSource::Command, "lspci"),
            }],
            Vec::new(),
        );
        let json = serde_json::to_string(&report).expect("report should serialize");
        let decoded: Report = serde_json::from_str(&json).expect("report should deserialize");

        assert_eq!(decoded, report);
=======
    use crate::facts::{Category, Fact, FactBuilder, FactSource, ObservationMethod};
    use crate::ids::{collector, fact};

    use super::{Report, SCHEMA};

    #[test]
    fn report_uses_v2_schema() {
        let report = Report::new(FactBuilder::new().build(), Vec::new(), Vec::new());
        assert_eq!(report.schema, SCHEMA);
    }

    #[test]
    fn sanitization_happens_on_finalized_report() {
        let mut builder = FactBuilder::new();
        builder.append([Fact::observed(
            fact::DESKTOP,
            Category::System,
            "/home/alice/.config",
            FactSource {
                collector: collector::SYSTEM,
                method: ObservationMethod::Environment("TEST".to_owned()),
            },
        )]);
        let report = Report::new(builder.build(), Vec::new(), Vec::new()).public_safe(false);

        assert_eq!(report.facts[0].value, "$HOME/.config");
    }

    #[test]
    fn sanitization_works_without_home_environment() {
        let mut builder = FactBuilder::new();
        builder.append([Fact::observed(
            fact::DESKTOP,
            Category::System,
            "/home/alice/.config",
            FactSource {
                collector: collector::SYSTEM,
                method: ObservationMethod::Environment("TEST".to_owned()),
            },
        )]);
        let report = Report::new(builder.build(), Vec::new(), Vec::new()).public_safe(false);

        assert_eq!(report.facts[0].value, "$HOME/.config");
>>>>>>> 2109919 (v0.2.0)
    }
}
