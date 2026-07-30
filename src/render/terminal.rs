//! Terminal (human-readable) report renderer.
//!
//! Output groups facts and checks by category, prefixes each check with a status icon,
//! and prints a summary footer.

use std::collections::BTreeMap;

use crate::model::{Category, Check, Confidence, Report, Status, Trace};

const RULE_CHAR: char = '━';
const RULE_LEN: usize = 60;

pub fn print_report(report: &Report, sanitize: bool, debug: bool) {
    let home = home_dir_string();
    let rule: String = std::iter::repeat(RULE_CHAR).take(RULE_LEN).collect();

    // Header
    println!("{rule}");
    println!(
        " GameDoc {}  ·  Linux Gaming Diagnostics",
        report.version
    );
    println!("{rule}");

    if report.facts.is_empty() && report.checks.is_empty() && report.problems.is_empty() {
        println!("\n No diagnostic data available.\n");
        println!("{rule}");
        return;
    }

    // Print Facts
    if !report.facts.is_empty() {
        println!("\n ── Facts ───────────────────────────────────────────────");
        for fact in &report.facts {
            let name = display_name(&fact.id);
            let val = sanitize_text(&fact.value, sanitize, home.as_deref());
            println!("    {name:<22} {val}");
            if debug {
                print_trace(fact.evidence.trace.as_ref(), sanitize, home.as_deref());
            }
        }
    }

    // Print Checks
    let grouped = group_by_category(&report.checks);
    for (category, checks) in &grouped {
        println!();
        println!(" {}", category_label(*category));
        for check in checks {
            print_check_line(check, sanitize, home.as_deref());
            if debug {
                print_trace(check.evidence.trace.as_ref(), sanitize, home.as_deref());
            }
        }
    }

    // Print detailed issues
    let issues: Vec<&&Check> = grouped
        .values()
        .flatten()
        .filter(|c| c.status != Status::Healthy || c.confidence == Confidence::Unknown)
        .collect();

    if !issues.is_empty() {
        println!();
        println!(" ── Issues ──────────────────────────────────────────────");
        for check in issues {
            print_issue_detail(check, sanitize, home.as_deref());
        }
    }

    // Print synthesized problems
    if !report.problems.is_empty() {
        println!();
        println!(" ── Synthesized Problems ────────────────────────────────");
        for problem in &report.problems {
            println!("\n   ✗ {}", display_name(&problem.id));
            let desc = sanitize_text(&problem.description, sanitize, home.as_deref());
            println!("        {desc}");
            if let Some(rec) = &problem.recommendation {
                let r = sanitize_text(rec, sanitize, home.as_deref());
                println!("        → {r}");
            }
        }
    }

    // Footer
    let total = report.checks.len();
    let healthy = report
        .checks
        .iter()
        .filter(|c| c.status == Status::Healthy && c.confidence != Confidence::Unknown)
        .count();
    let warnings = report
        .checks
        .iter()
        .filter(|c| c.status == Status::Warning)
        .count();
    let problems = report
        .checks
        .iter()
        .filter(|c| c.status == Status::Problem)
        .count();
    let unknowns = report
        .checks
        .iter()
        .filter(|c| c.confidence == Confidence::Unknown)
        .count();

    println!();
    println!("{rule}");
    println!(
        " {total} checks  ·  {healthy} ✓ healthy  ·  {warnings} ⚠ warnings  ·  {problems} ✗ problems  ·  {unknowns} ? unknown"
    );
    println!("{rule}");
}

fn print_check_line(check: &Check, sanitize: bool, home: Option<&str>) {
    let icon = status_icon(check.status, check.confidence);
    let name = display_name(&check.id);
    let summary = sanitize_text(&check.summary, sanitize, home);

    println!("   {icon}  {name:<22} {summary}");
}

fn print_issue_detail(check: &Check, sanitize: bool, home: Option<&str>) {
    let icon = status_icon(check.status, check.confidence);
    let summary = sanitize_text(&check.summary, sanitize, home);

    println!("\n   {icon}  {} — {summary}", check.id);

    for detail in &check.details {
        let d = sanitize_text(detail, sanitize, home);
        println!("        {d}");
    }

    if let Some(rec) = &check.recommendation {
        let r = sanitize_text(rec, sanitize, home);
        println!("        → {r}");
    }
}

fn print_trace(trace: Option<&Trace>, _sanitize: bool, _home: Option<&str>) {
    if let Some(t) = trace {
        let exit_str = match t.exit_code {
            Some(code) => code.to_string(),
            None => "signal".to_owned(),
        };
        println!(
            "       ↳ [DEBUG] exited with {exit_str} in {} ms",
            t.duration_ms
        );
    }
}

fn status_icon(status: Status, confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::Unknown => "?",
        Confidence::Error => "✗",
        _ => match status {
            Status::Healthy => "✓",
            Status::Warning => "⚠",
            Status::Problem => "✗",
        },
    }
}

fn display_name(id: &str) -> String {
    let part = id.split('.').next_back().unwrap_or(id);
    let part = part.replace('_', " ");
    let mut chars = part.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn category_label(category: Category) -> &'static str {
    match category {
        Category::System => "System",
        Category::Gpu => "GPU",
        Category::Steam => "Steam",
        Category::MangoHud => "MangoHUD",
        Category::GameMode => "GameMode",
    }
}

fn group_by_category(checks: &[Check]) -> BTreeMap<Category, Vec<&Check>> {
    let mut map: BTreeMap<Category, Vec<&Check>> = BTreeMap::new();
    for check in checks {
        map.entry(check.category).or_default().push(check);
    }
    map
}

#[must_use]
pub fn sanitize_text(text: &str, sanitize: bool, home: Option<&str>) -> String {
    if !sanitize {
        return text.to_owned();
    }
    match home {
        Some(h) if !h.is_empty() && text.contains(h) => text.replace(h, "<home>"),
        _ => text.to_owned(),
    }
}

fn home_dir_string() -> Option<String> {
    std::env::var("HOME").ok().filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{display_name, group_by_category, sanitize_text, status_icon};
    use crate::model::{Category, Check, Confidence, Evidence, EvidenceSource, Status};

    fn make_check(id: &str, status: Status, confidence: Confidence) -> Check {
        Check {
            id: id.to_owned(),
            category: Category::System,
            status,
            confidence,
            summary: "Test summary".to_owned(),
            details: Vec::new(),
            recommendation: None,
            evidence: Evidence::new(EvidenceSource::FileSystem, "test"),
        }
    }

    #[test]
    fn sanitize_replaces_home_when_enabled() {
        let result = sanitize_text("/home/user/file", true, Some("/home/user"));
        assert_eq!(result, "<home>/file");
    }

    #[test]
    fn sanitize_is_noop_when_disabled() {
        let result = sanitize_text("/home/user/file", false, Some("/home/user"));
        assert_eq!(result, "/home/user/file");
    }

    #[test]
    fn sanitize_is_noop_when_home_is_none() {
        let result = sanitize_text("/home/user/file", true, None);
        assert_eq!(result, "/home/user/file");
    }

    #[test]
    fn sanitize_handles_text_without_home() {
        let result = sanitize_text("some other text", true, Some("/home/user"));
        assert_eq!(result, "some other text");
    }

    #[test]
    fn status_icon_healthy_detected() {
        assert_eq!(status_icon(Status::Healthy, Confidence::Detected), "✓");
    }

    #[test]
    fn status_icon_warning_detected() {
        assert_eq!(status_icon(Status::Warning, Confidence::Detected), "⚠");
    }

    #[test]
    fn status_icon_unknown_confidence_overrides_status() {
        assert_eq!(status_icon(Status::Healthy, Confidence::Unknown), "?");
        assert_eq!(status_icon(Status::Warning, Confidence::Unknown), "?");
    }

    #[test]
    fn status_icon_error_confidence() {
        assert_eq!(status_icon(Status::Healthy, Confidence::Error), "✗");
    }

    #[test]
    fn display_name_from_dotted_id() {
        assert_eq!(display_name("system.kernel_version"), "Kernel version");
    }

    #[test]
    fn display_name_from_simple_id() {
        assert_eq!(display_name("architecture"), "Architecture");
    }

    #[test]
    fn group_by_category_returns_only_present_categories() {
        let checks = vec![
            make_check(
                "system.kernel_version",
                Status::Healthy,
                Confidence::Detected,
            ),
            make_check("system.memory", Status::Healthy, Confidence::Detected),
        ];
        let grouped = group_by_category(&checks);
        assert_eq!(grouped.len(), 1);
        assert!(grouped.contains_key(&Category::System));
    }

    #[test]
    fn group_by_category_preserves_all_checks() {
        let checks = vec![
            make_check(
                "system.kernel_version",
                Status::Healthy,
                Confidence::Detected,
            ),
            make_check("system.memory", Status::Healthy, Confidence::Detected),
        ];
        let grouped = group_by_category(&checks);
        let total: usize = grouped.values().map(Vec::len).sum();
        assert_eq!(total, 2);
    }
}
