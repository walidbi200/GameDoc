//! Colourful terminal rendering for finalized reports.

use std::fmt::Write as _;

use crate::ids::fact;
use crate::model::Report;
use crate::render::color;

// ── Config ────────────────────────────────────────────────────────────────────

pub struct TerminalConfig {
    pub summary: bool,
    pub verbose: bool,
    pub ascii: bool,
    pub duration_ms: u64,
}

// ── Layout constants ──────────────────────────────────────────────────────────

/// Width of the key column in `key  value` rows.
const LABEL_WIDTH: usize = 12;

/// Width of the score progress bar in blocks.
const SCORE_BAR_WIDTH: usize = 30;

/// Unicode divider — 46 chars, fits an 80-col terminal with comfortable margin.
const DIVIDER: &str = "──────────────────────────────────────────────";
const DIVIDER_ASCII: &str = "----------------------------------------------";

// ── Icons ─────────────────────────────────────────────────────────────────────

struct Icons {
    ascii: bool,
}

impl Icons {
    fn ok(&self) -> &'static str {
        if self.ascii {
            "[OK]"
        } else {
            "✓"
        }
    }
    fn warn(&self) -> &'static str {
        if self.ascii {
            "[!]"
        } else {
            "⚠"
        }
    }
    fn err(&self) -> &'static str {
        if self.ascii {
            "[X]"
        } else {
            "✗"
        }
    }
    fn divider(&self) -> &'static str {
        if self.ascii {
            DIVIDER_ASCII
        } else {
            DIVIDER
        }
    }
    fn recommendations(&self) -> &'static str {
        if self.ascii {
            "Recommendations"
        } else {
            "💡 Recommendations"
        }
    }
    fn checks_skipped(&self) -> &'static str {
        if self.ascii {
            "Checks Skipped"
        } else {
            "⊘ Checks Skipped"
        }
    }
    fn raw_facts(&self) -> &'static str {
        if self.ascii {
            "Raw Facts"
        } else {
            "📦 Raw Facts"
        }
    }
}

// ── Low-level helpers ─────────────────────────────────────────────────────────

fn section(title: &str, icons: &Icons, out: &mut String) {
    let _ = writeln!(out, "\n{}", color::bold(title));
    let _ = writeln!(out, "{}", color::gray(icons.divider()));
}

fn row(label: &str, value_ansi: &str, out: &mut String) {
    let padded = format!("{label:<LABEL_WIDTH$}");
    let _ = writeln!(out, "{}  {}", color::cyan(&padded), value_ansi);
}

fn ok_str(icons: &Icons) -> String {
    color::green(icons.ok())
}

fn warn_str(icons: &Icons, text: &str) -> String {
    format!("{} {}", color::yellow(icons.warn()), text)
}

fn err_str(icons: &Icons, text: &str) -> String {
    format!("{} {}", color::red(icons.err()), text)
}

// ── Fact extraction ───────────────────────────────────────────────────────────

fn get(report: &Report, id: crate::ids::FactId) -> Option<String> {
    report
        .facts
        .iter()
        .find(|f| f.id == id)
        .map(|f| match &f.value {
            serde_json::Value::String(s) => s.clone(),
            v => v.to_string(),
        })
}

fn get_all(report: &Report, id: crate::ids::FactId) -> impl Iterator<Item = String> + '_ {
    report
        .facts
        .iter()
        .filter(move |f| f.id == id)
        .map(|f| match &f.value {
            serde_json::Value::String(s) => s.clone(),
            v => v.to_string(),
        })
}

// ── Section renderers ─────────────────────────────────────────────────────────

fn render_header(report: &Report, _config: &TerminalConfig, icons: &Icons, out: &mut String) {
    let title = if icons.ascii {
        format!("GameDoc v{}", report.gamedoc_version)
    } else {
        format!("🎮 GameDoc v{}", report.gamedoc_version)
    };
    let _ = writeln!(out, "{}", color::bold(&title));
    let _ = writeln!(out, "{}", color::gray(icons.divider()));
}

fn render_score_bar(score: u32) -> String {
    let bar_width = u32::try_from(SCORE_BAR_WIDTH).unwrap_or(u32::MAX);
    let filled = usize::try_from((score * bar_width + 50) / 100)
        .unwrap_or(SCORE_BAR_WIDTH)
        .min(SCORE_BAR_WIDTH);
    let empty = SCORE_BAR_WIDTH - filled;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if score >= 80 {
        color::green(&bar)
    } else if score >= 60 {
        color::yellow(&bar)
    } else {
        color::red(&bar)
    }
}

fn score_verdict(score: u32, icons: &Icons) -> String {
    let (icon_fn, text): (fn(&Icons) -> &'static str, &'static str) = match score {
        95..=100 => (Icons::ok, "Excellent Linux Gaming Environment"),
        80..=94 => (Icons::ok, "Ready for Linux Gaming"),
        60..=79 => (Icons::warn, "Gaming Ready — Minor Issues"),
        40..=59 => (Icons::warn, "Needs Attention"),
        _ => (Icons::err, "Gaming Environment Has Critical Issues"),
    };

    let icon = icon_fn(icons);
    match score {
        80..=100 => format!("{} {}", color::green(icon), color::green(text)),
        60..=79 => format!("{} {}", color::yellow(icon), color::yellow(text)),
        _ => format!("{} {}", color::red(icon), color::red(text)),
    }
}

fn render_dashboard(report: &Report, icons: &Icons, out: &mut String) {
    let score = report.gaming_score();

    // ── Gaming Score hero block ───────────────────────────────────────────────
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", color::bold("Gaming Score"));
    let _ = writeln!(out);

    let bar = render_score_bar(score);
    let score_label = if score >= 80 {
        color::green(&format!("{score} / 100"))
    } else if score >= 60 {
        color::yellow(&format!("{score} / 100"))
    } else {
        color::red(&format!("{score} / 100"))
    };
    let _ = writeln!(out, "{bar}  {score_label}");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", score_verdict(score, icons));

    // ── System ────────────────────────────────────────────────────────────────
    section("System", icons, out);

    let os = get(report, fact::DISTRO_PRETTY_NAME).or_else(|| get(report, fact::DISTRO_NAME));
    if let Some(v) = os {
        row("OS", &v, out);
    }

    if let Some(kernel) = get(report, fact::KERNEL_VERSION) {
        row("Kernel", &kernel, out);
    }

    // Collect unique CPU models
    let models: std::collections::BTreeSet<_> = get_all(report, fact::CPU_MODEL).collect();
    if !models.is_empty() {
        let joined = models.into_iter().collect::<Vec<_>>().join(", ");
        row("CPU", &joined, out);
    }

    // Collect unique GPU names
    let mut gpus: Vec<_> = get_all(report, fact::GPU_DEVICE).collect();
    gpus.dedup();
    for gpu in &gpus {
        row("GPU", gpu, out);
    }

    if let Some(driver) = get(report, fact::OPENGL_VERSION) {
        row("Driver", &driver, out);
    }

    if let Some(governor) = get(report, fact::CPU_GOVERNOR) {
        let governor_str = if governor == "powersave" {
            warn_str(icons, &format!("{governor} — may hurt gaming performance"))
        } else {
            governor.clone()
        };
        row("Governor", &governor_str, out);
    }

    if let Some(session) = get(report, fact::SESSION_TYPE) {
        row("Session", &session, out);
    }

    // ── Gaming ────────────────────────────────────────────────────────────────
    section("Gaming", icons, out);

    let steam_ok = get_all(report, fact::STEAM_INSTALLATION).count() > 0;
    let steam_str = if steam_ok {
        ok_str(icons)
    } else {
        err_str(icons, "Not found")
    };
    row("Steam", &steam_str, out);

    let gamemode_running = get(report, fact::GAMEMODE_DAEMON_RUNNING) == Some("true".to_string());
    let gamemode_available =
        get(report, fact::GAMEMODE_DAEMON_AVAILABLE) == Some("true".to_string());
    let gamemode_str = if gamemode_running || gamemode_available {
        ok_str(icons)
    } else {
        warn_str(icons, "Not found")
    };
    row("GameMode", &gamemode_str, out);

    let gamescope = report
        .facts
        .iter()
        .any(|f| f.id == fact::GAMESCOPE_INSTALLATION && f.value.as_bool() == Some(true));
    let gamescope_str = if gamescope {
        ok_str(icons)
    } else {
        warn_str(icons, "Not found")
    };
    row("Gamescope", &gamescope_str, out);

    let mangohud = report
        .facts
        .iter()
        .any(|f| f.id == fact::MANGOHUD_INSTALLATION);
    let mangohud_str = if mangohud {
        ok_str(icons)
    } else {
        warn_str(icons, "Not found")
    };
    row("MangoHud", &mangohud_str, out);

    let opengl_ok = get(report, fact::OPENGL_RENDERER).is_some();
    let opengl_str = if opengl_ok {
        ok_str(icons)
    } else {
        err_str(icons, "Not found")
    };
    row("OpenGL", &opengl_str, out);

    let vulkan_ok = get(report, fact::VULKAN_AVAILABLE) == Some("true".to_string());
    let vulkan_str = if vulkan_ok {
        ok_str(icons)
    } else {
        err_str(icons, "Not found")
    };
    row("Vulkan", &vulkan_str, out);
}

fn render_recommendations(report: &Report, icons: &Icons, out: &mut String) {
    if report.problems.is_empty() {
        return;
    }

    let distro = report.distro_family();
    section(icons.recommendations(), icons, out);

    for (i, problem) in report.problems.iter().enumerate() {
        let _ = writeln!(out, "{}. {}", i + 1, color::bold(&problem.summary));

        if let Some(rec) = &problem.recommendation {
            if !rec.text.is_empty() {
                let _ = writeln!(out);
                let _ = writeln!(
                    out,
                    "   {}    {}",
                    color::cyan("Reason"),
                    color::gray(&rec.text)
                );
            }

            let fix = rec.for_distro(distro);
            if !fix.is_empty() && fix != rec.text {
                let _ = writeln!(out);
                let _ = writeln!(out, "   {}   {}", color::cyan("Command"), color::dim(fix));
            }
        }

        for detail in &problem.details {
            let _ = writeln!(out, "   {}", color::dim(detail));
        }

        out.push('\n');
    }
}

fn render_checks_skipped(report: &Report, icons: &Icons, out: &mut String) {
    if report.checks.is_empty() {
        return;
    }
    section(icons.checks_skipped(), icons, out);
    for diagnostic in &report.checks {
        let _ = writeln!(
            out,
            "  {}  {} — {}",
            color::dim(icons.warn()),
            diagnostic.collector,
            diagnostic.message
        );
    }
}

fn render_footer(report: &Report, config: &TerminalConfig, icons: &Icons, out: &mut String) {
    let _ = writeln!(out, "\n{}", color::gray(icons.divider()));
    let _ = writeln!(
        out,
        "{}",
        color::gray(&format!(
            "{} ms  ·  {} facts  ·  {} issues  ·  {} skipped",
            config.duration_ms,
            report.facts.len(),
            report.problems.len(),
            report.checks.len(),
        ))
    );
}

fn render_raw_facts(report: &Report, out: &mut String) {
    if report.facts.is_empty() {
        out.push_str("No facts were collected.\n");
    } else {
        for fact in &report.facts {
            let value = match &fact.value {
                serde_json::Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            let _ = writeln!(
                out,
                "{}  {}  {}",
                color::dim(fact.id.as_str()),
                value,
                color::gray(&format!("({})", fact.confidence))
            );
        }
    }
}

// ── Top-level entry point ─────────────────────────────────────────────────────

#[must_use]
pub fn render_report(report: &Report, config: &TerminalConfig) -> String {
    let mut out = String::new();
    let icons = Icons {
        ascii: config.ascii,
    };

    render_header(report, config, &icons, &mut out);
    render_dashboard(report, &icons, &mut out);

    if config.summary {
        render_recommendations(report, &icons, &mut out);
        render_footer(report, config, &icons, &mut out);
        return out;
    }

    render_recommendations(report, &icons, &mut out);
    render_checks_skipped(report, &icons, &mut out);
    render_footer(report, config, &icons, &mut out);

    if config.verbose {
        let _ = writeln!(&mut out);
        section(icons.raw_facts(), &icons, &mut out);
        render_raw_facts(report, &mut out);
    }

    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::facts::FactBuilder;
    use crate::model::Report;

    use super::{render_report, TerminalConfig};

    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' {
                    in_escape = false;
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    fn ascii_config() -> TerminalConfig {
        TerminalConfig {
            summary: false,
            verbose: false,
            ascii: true,
            duration_ms: 147,
        }
    }

    #[test]
    fn empty_report_renders_header_and_health() {
        let report = Report::new(FactBuilder::new().build(), Vec::new(), Vec::new());
        let rendered = render_report(&report, &ascii_config());
        let plain = strip_ansi(&rendered);
        assert!(plain.contains("GameDoc v0.3.0"), "missing title");
        assert!(plain.contains("Gaming Score"), "missing gaming score");
        assert!(plain.contains("147 ms"), "missing footer");
    }

    #[test]
    fn distro_aware_recommendation_picks_correct_platform() {
        use crate::facts::{Category, Fact, FactSource, ObservationMethod};
        use crate::ids::{collector, fact, problem};
        use crate::model::{Problem, Recommendation, Severity};
        use std::collections::BTreeMap;

        let mut builder = FactBuilder::new();
        builder.append([Fact::observed(
            fact::DISTRO_FAMILY,
            Category::Distro,
            "fedora",
            FactSource {
                collector: collector::DISTRO,
                method: ObservationMethod::File("/etc/os-release".to_owned()),
            },
        )]);

        let problem = Problem {
            id: problem::VULKAN_UNAVAILABLE,
            category: Category::Gpu,
            severity: Severity::Critical,
            summary: "Vulkan unavailable".to_owned(),
            details: Vec::new(),
            recommendation: Some(Recommendation {
                text: "Vulkan is not available.".to_owned(),
                platform_specific: BTreeMap::from([(
                    "fedora".to_owned(),
                    "dnf install vulkan-tools".to_owned(),
                )]),
                difficulty: 1,
                est_minutes: 1,
                requires_root: true,
                requires_reboot: false,
                docs_url: None,
            }),
        };

        let report = Report::new(builder.build(), vec![problem], Vec::new());
        let rendered = render_report(&report, &ascii_config());
        assert!(
            rendered.contains("dnf install vulkan-tools"),
            "fix command missing"
        );
        assert!(
            rendered.contains("Vulkan is not available."),
            "reason missing"
        );
    }

    #[test]
    fn score_100_maps_to_excellent_verdict() {
        let report = Report::new(FactBuilder::new().build(), Vec::new(), Vec::new());
        // Empty report with no checks has score 100
        let rendered = render_report(&report, &ascii_config());
        let plain = strip_ansi(&rendered);
        assert!(
            plain.contains("Excellent Linux Gaming Environment"),
            "expected excellent verdict, got:\n{plain}"
        );
    }
}
