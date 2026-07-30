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
}

impl Report {
    #[must_use]
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
}

#[cfg(test)]
mod tests {
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
    }
}
