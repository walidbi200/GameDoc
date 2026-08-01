//! Stable JSON rendering for finalized reports.

use anyhow::{Context, Result};

use crate::model::Report;

use serde::Serialize;

#[derive(Serialize)]
struct ReportWithSchema<'a> {
    schema_url: &'a str,
    #[serde(flatten)]
    report: &'a Report,
}

/// Serialize a finalized report using the public JSON schema.
///
/// # Errors
///
/// Returns an error if the report cannot be serialized.
pub fn render_report(report: &Report) -> Result<String> {
    let wrapper = ReportWithSchema {
        schema_url: "https://raw.githubusercontent.com/walidbi200/GameDoc/main/schema/gamedoc.v2.schema.json",
        report,
    };
    let mut json =
        serde_json::to_string_pretty(&wrapper).context("failed to serialize report to JSON")?;
    json.push('\n');
    Ok(json)
}

#[cfg(test)]
mod tests {
    use crate::facts::FactBuilder;
    use crate::model::Report;

    use super::render_report;

    #[test]
    fn empty_report_matches_semantic_golden() {
        let report = Report::new(FactBuilder::new().build(), Vec::new(), Vec::new());
        let output = render_report(&report).unwrap();

        assert_eq!(
            output,
            concat!(
                "{\n",
                "  \"schema_url\": \"https://raw.githubusercontent.com/walidbi200/GameDoc/main/schema/gamedoc.v2.schema.json\",\n",
                "  \"schema\": \"gamedoc.v2\",\n",
                "  \"gamedoc_version\": \"",
                env!("CARGO_PKG_VERSION"),
                "\",\n",
                "  \"facts\": [],\n",
                "  \"problems\": [],\n",
                "  \"checks\": []\n",
                "}\n"
            )
        );
    }
}
