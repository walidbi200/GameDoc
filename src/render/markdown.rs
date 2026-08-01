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
        Category::Gpu => "GPU",
        Category::Steam => "Steam",
        Category::MangoHud => "MangoHUD",
        Category::GameMode => "GameMode",
    }
}

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
                "**GameDoc version:** 0.3.0  \n",
                "**Schema:** `gamedoc.v2`\n\n",
                "## Facts\n\n",
                "_No facts were collected._\n\n",
                "## Problems\n\n",
                "_No problems detected from available evidence._\n"
            )
        );
    }
}
