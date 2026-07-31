<<<<<<< HEAD
//! GitHub-flavored Markdown report renderer.
//!
//! Output is designed to paste well into GitHub issues, Reddit posts,
//! and Discord code blocks.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::model::{Category, Check, Confidence, Report, Status};

/// Render `report` as a GitHub-flavored Markdown string.
///
/// The returned string ends with a trailing newline.
#[must_use]
pub fn render_report(report: &Report) -> String {
    let mut out = String::new();

    out.push_str("# GameDoc Report\n\n");
    let _ = writeln!(out, "**GameDoc version:** {}  ", report.version);
    let _ = writeln!(out, "**Schema:** {}  ", report.schema);

    if report.facts.is_empty() && report.checks.is_empty() && report.problems.is_empty() {
        out.push_str("\n_No diagnostic data available._\n");
        return out;
    }

    if !report.facts.is_empty() {
        out.push_str("\n## Facts\n\n");
        out.push_str("| System | Property | Value |\n");
        out.push_str("|--------|----------|-------|\n");
        for fact in &report.facts {
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                category_heading(fact.category),
                display_name(&fact.id),
                fact.value
            );
        }
    }

    if !report.checks.is_empty() {
        // Group by category in enum declaration order (BTreeMap uses Ord).
        let grouped = group_by_category(&report.checks);

        for (category, checks) in &grouped {
            out.push('\n');
            let _ = write!(out, "## {}\n\n", category_heading(*category));

            // Summary table for the category.
            out.push_str("| Check | Status | Confidence |\n");
            out.push_str("|-------|--------|------------|\n");
            for check in checks {
                let _ = writeln!(
                    out,
                    "| {} | {} {} | {} |",
                    display_name(&check.id),
                    status_emoji(check.status, check.confidence),
                    status_label(check.status, check.confidence),
                    confidence_label(check.confidence),
                );
            }
        }

        // Detail sections for all checks that have details or recommendations.
        let detailed: Vec<&&Check> = grouped
            .values()
            .flatten()
            .filter(|c| !c.details.is_empty() || c.recommendation.is_some())
            .collect();

        if !detailed.is_empty() {
            out.push_str("\n---\n");
            for check in detailed {
                out.push('\n');
                let _ = write!(out, "### `{}`\n\n", check.id);
                for detail in &check.details {
                    let _ = writeln!(out, "- {detail}");
                }
                if let Some(rec) = &check.recommendation {
                    out.push('\n');
                    let _ = writeln!(out, "**Recommendation:** {rec}");
                }
            }
        }
    }

    if !report.problems.is_empty() {
        out.push_str("\n## Synthesized Problems\n\n");
        for problem in &report.problems {
            let _ = writeln!(out, "### ❌ {}\n", display_name(&problem.id));
            let _ = writeln!(out, "{}", problem.description);
            if let Some(rec) = &problem.recommendation {
                let _ = writeln!(out, "\n**Recommendation:** {rec}");
            }
        }
    }

    // Ensure trailing newline.
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn group_by_category(checks: &[Check]) -> BTreeMap<Category, Vec<&Check>> {
    let mut map: BTreeMap<Category, Vec<&Check>> = BTreeMap::new();
    for check in checks {
        map.entry(check.category).or_default().push(check);
    }
    map
}

fn category_heading(category: Category) -> &'static str {
    match category {
        Category::System => "System",
=======
//! GitHub-flavored Markdown rendering for finalized reports.

use std::fmt::Write as _;

use crate::facts::{Category, Fact};
use crate::model::{Problem, Report};

#[must_use]
pub fn render_report(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("# GameDoc Report\n\n");
    let _ = write!(
        out,
        "**GameDoc version:** {}  \n**Schema:** `{}`\n\n",
        report.gamedoc_version, report.schema
    );

    out.push_str("## Facts\n\n");
    if report.facts.is_empty() {
        out.push_str("_No facts were collected._\n\n");
    } else {
        out.push_str("| Category | Fact | Value | Confidence |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for fact in &report.facts {
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} |",
                category_label(fact.category),
                fact.id,
                markdown_value(fact),
                fact.confidence
            );
        }
        out.push('\n');
    }

    out.push_str("## Problems\n\n");
    if report.problems.is_empty() {
        out.push_str("_No problems detected from available evidence._\n");
    } else {
        let distro = report.distro_family();
        for problem in &report.problems {
            render_problem(&mut out, problem, distro);
        }
    }

    if !report.checks.is_empty() {
        out.push_str("\n## Incomplete Observations\n\n");
        for diagnostic in &report.checks {
            let _ = writeln!(
                out,
                "- **{}** (`{}`): {}",
                diagnostic.collector, diagnostic.operation, diagnostic.message
            );
        }
    }

    out
}

fn render_problem(out: &mut String, problem: &Problem, distro_family: Option<&str>) {
    let _ = writeln!(out, "### {}: {}\n", problem.severity, problem.summary);
    let _ = writeln!(out, "- **ID:** `{}`", problem.id);
    let _ = writeln!(out, "- **Category:** {}", category_label(problem.category));
    for detail in &problem.details {
        let _ = writeln!(out, "- {detail}");
    }
    if let Some(recommendation) = &problem.recommendation {
        let _ = writeln!(
            out,
            "\n**Recommendation:** {}",
            recommendation.for_distro(distro_family)
        );
    }
    out.push('\n');
}

fn markdown_value(fact: &Fact) -> String {
    let value = match &fact.value {
        serde_json::Value::String(value) => value.clone(),
        value => value.to_string(),
    };
    value.replace('|', "\\|").replace('\n', " ")
}

fn category_label(category: Category) -> &'static str {
    match category {
        Category::System => "System",
        Category::Cpu => "CPU",
        Category::Distro => "Distro",
>>>>>>> 2109919 (v0.2.0)
        Category::Gpu => "GPU",
        Category::Steam => "Steam",
        Category::MangoHud => "MangoHUD",
        Category::GameMode => "GameMode",
    }
}

<<<<<<< HEAD
fn display_name(id: &str) -> String {
    let part = id.split('.').next_back().unwrap_or(id);
    let part = part.replace('_', " ");
    let mut chars = part.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn status_emoji(status: Status, confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::Unknown => "❓",
        Confidence::Error => "❌",
        _ => match status {
            Status::Healthy => "✅",
            Status::Warning => "⚠️",
            Status::Problem => "❌",
        },
    }
}

fn status_label(status: Status, confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::Unknown => "Unknown",
        Confidence::Error => "Error",
        _ => match status {
            Status::Healthy => "Healthy",
            Status::Warning => "Warning",
            Status::Problem => "Problem",
        },
    }
}

fn confidence_label(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::Detected => "Detected",
        Confidence::NotDetected => "Not detected",
        Confidence::Unknown => "Unknown",
        Confidence::Error => "Error",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::render_report;
    use crate::model::{
        Category, Check, Confidence, Evidence, EvidenceSource, Fact, Report, Status,
    };

    fn make_check(id: &str, status: Status, confidence: Confidence) -> Check {
        Check {
            id: id.to_owned(),
            category: Category::System,
            status,
            confidence,
            summary: format!("Summary for {id}"),
            details: Vec::new(),
            recommendation: None,
            evidence: Evidence::new(EvidenceSource::FileSystem, "test"),
        }
    }

    #[test]
    fn empty_report_renders_placeholder() {
        let report = Report::new("0.1.0", Vec::new(), Vec::new(), Vec::new());
        let md = render_report(&report);
        assert!(md.contains("No diagnostic data available"));
        assert!(md.contains("0.1.0"));
    }

    #[test]
    fn report_with_checks_contains_check_ids() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![make_check(
                "system.kernel_version",
                Status::Healthy,
                Confidence::Detected,
            )],
            Vec::new(),
        );
        let md = render_report(&report);
        assert!(
            md.contains("system.kernel_version") || md.contains("Kernel version"),
            "check id or name must appear in markdown output"
        );
    }

    #[test]
    fn report_with_facts_contains_fact_table() {
        let report = Report::new(
            "0.1.0",
            vec![Fact {
                id: "system.kernel_version".to_owned(),
                category: Category::System,
                value: "6.9.0".to_owned(),
                evidence: Evidence::new(EvidenceSource::FileSystem, "test"),
            }],
            Vec::new(),
            Vec::new(),
        );
        let md = render_report(&report);
        assert!(md.contains("## Facts"));
        assert!(md.contains("6.9.0"));
    }

    #[test]
    fn healthy_check_uses_checkmark_emoji() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![make_check(
                "system.memory",
                Status::Healthy,
                Confidence::Detected,
            )],
            Vec::new(),
        );
        let md = render_report(&report);
        assert!(md.contains("✅"), "healthy checks must use ✅");
    }

    #[test]
    fn problem_check_uses_cross_emoji() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![make_check(
                "gpu.vulkan",
                Status::Problem,
                Confidence::NotDetected,
            )],
            Vec::new(),
        );
        let md = render_report(&report);
        assert!(md.contains("❌"), "problem checks must use ❌");
    }

    #[test]
    fn unknown_confidence_uses_question_emoji() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![make_check(
                "system.memory",
                Status::Warning,
                Confidence::Unknown,
            )],
            Vec::new(),
        );
        let md = render_report(&report);
        assert!(md.contains("❓"), "unknown confidence must use ❓");
    }

    #[test]
    fn recommendation_is_rendered_when_present() {
        let mut check = make_check("system.memory", Status::Warning, Confidence::Unknown);
        check.recommendation = Some("Install more RAM (just kidding).".to_owned());

        let report = Report::new("0.1.0", Vec::new(), vec![check], Vec::new());
        let md = render_report(&report);
        assert!(
            md.contains("Install more RAM"),
            "recommendation must appear in markdown"
        );
    }

    #[test]
    fn category_heading_system_appears() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![make_check(
                "system.kernel_version",
                Status::Healthy,
                Confidence::Detected,
            )],
            Vec::new(),
        );
        let md = render_report(&report);
        assert!(
            md.contains("## System"),
            "System category heading must appear"
        );
    }

    #[test]
    fn output_ends_with_newline() {
        let report = Report::new("0.1.0", Vec::new(), Vec::new(), Vec::new());
        assert!(render_report(&report).ends_with('\n'));
    }

    #[test]
    fn summary_table_has_correct_columns() {
        let report = Report::new(
            "0.1.0",
            Vec::new(),
            vec![make_check(
                "system.architecture",
                Status::Healthy,
                Confidence::Detected,
            )],
            Vec::new(),
        );
        let md = render_report(&report);
        // All three table columns must be present.
        assert!(md.contains("Status"), "table must have Status column");
        assert!(
            md.contains("Confidence"),
            "table must have Confidence column"
=======
#[cfg(test)]
mod tests {
    use crate::facts::FactBuilder;
    use crate::model::Report;

    use super::render_report;

    #[test]
    fn empty_report_matches_presentation_golden() {
        let report = Report::new(FactBuilder::new().build(), Vec::new(), Vec::new());

        assert_eq!(
            render_report(&report),
            concat!(
                "# GameDoc Report\n\n",
                "**GameDoc version:** 0.2.0  \n",
                "**Schema:** `gamedoc.v2`\n\n",
                "## Facts\n\n",
                "_No facts were collected._\n\n",
                "## Problems\n\n",
                "_No problems detected from available evidence._\n"
            )
>>>>>>> 2109919 (v0.2.0)
        );
    }
}
