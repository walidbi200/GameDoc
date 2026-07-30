//! JSON renderer.
//!
//! Serializes a [`crate::model::Report`] as pretty-printed JSON.
//!
//! The schema is designed to be stable for downstream consumers, and is
//! versioned via the `schema` field. Consumers of the JSON output
//! should check `schema` before parsing.

use anyhow::{Context, Result};

use crate::model::Report;

/// Renders a report as a pretty-printed JSON string.
///
/// The caller is responsible for writing the output (e.g. via `print!`).
///
/// # Errors
///
/// Returns an error if serialization fails.  In practice, the `Report` model
/// is fully serializable, so this should not happen in normal operation.
pub fn render_report(report: &Report) -> Result<String> {
    let mut json =
        serde_json::to_string_pretty(report).context("failed to serialize report to JSON")?;
    // Ensure the output ends with a newline so that shell redirects produce
    // well-formed text files.
    json.push('\n');
    Ok(json)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::model::{Category, Check, Confidence, Evidence, EvidenceSource, Report, Status};

    use super::render_report;

    fn make_report_with_check() -> Report {
        Report::new(
            "0.1.0",
            Vec::new(),
            vec![Check {
                id: "system.kernel".to_owned(),
                category: Category::System,
                status: Status::Healthy,
                confidence: Confidence::Detected,
                summary: "Kernel is up to date".to_owned(),
                details: vec!["Running 6.9.0".to_owned()],
                recommendation: None,
                evidence: Evidence::new(EvidenceSource::FileSystem, "test"),
            }],
            Vec::new(),
        )
    }

    #[test]
    fn empty_report_is_valid_json() {
        let report = Report::new("0.1.0", Vec::new(), Vec::new(), Vec::new());
        let output = render_report(&report).expect("render should succeed");
        let parsed: Value = serde_json::from_str(&output).expect("output should be valid JSON");

        assert_eq!(parsed["schema"], "gamedoc.v1");
        assert_eq!(parsed["generator"], "gamedoc");
        assert_eq!(parsed["version"], "0.1.0");
        assert!(parsed["diagnostics"].as_array().unwrap().is_empty());
        assert!(parsed["facts"].as_array().unwrap().is_empty());
        assert!(parsed["problems"].as_array().unwrap().is_empty());
    }

    #[test]
    fn report_with_check_round_trips() {
        let report = make_report_with_check();
        let output = render_report(&report).unwrap();
        let parsed: Report = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed, report);
    }

    #[test]
    fn output_ends_with_newline() {
        let report = Report::new("0.1.0", Vec::new(), Vec::new(), Vec::new());
        let output = render_report(&report).unwrap();
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn check_status_serializes_as_snake_case() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![Check {
                id: "test".to_owned(),
                category: Category::System,
                status: Status::Warning,
                confidence: Confidence::Unknown,
                summary: "Test".to_owned(),
                details: Vec::new(),
                recommendation: None,
                evidence: Evidence::new(EvidenceSource::FileSystem, "FOO"),
            }],
            Vec::new(),
        );

        let output = render_report(&report).unwrap();
        let parsed: Value = serde_json::from_str(&output).unwrap();

        let status = parsed["diagnostics"][0]["status"]
            .as_str()
            .expect("status should be a string");
        assert_eq!(status, "warning");

        let confidence = parsed["diagnostics"][0]["confidence"]
            .as_str()
            .expect("confidence should be a string");
        assert_eq!(confidence, "unknown");
    }
}
