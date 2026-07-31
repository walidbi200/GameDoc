//! Orchestrates the one-way diagnostic pipeline.

use crate::collect;
use crate::model::Report;
use crate::probe::SystemProbe;
use crate::{rules, synthesize};

#[must_use]
pub fn build_report(system: &dyn SystemProbe) -> Report {
    let (facts, checks) = collect::collect_all(system);
    let outcomes = rules::evaluate_all(&facts);
    let problems = synthesize::synthesize(&outcomes);
    Report::new(facts, problems, checks)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::collect::command::CommandOutput;
    use crate::probe::FixtureSystemProbe;
    use crate::render;

    use super::build_report;

    #[test]
    fn fixture_pipeline_matches_json_semantic_golden() {
        let fixture = FixtureSystemProbe::new()
            .with_command("uname", &["-r"], output("6.9.0\n", 0))
            .with_command("uname", &["-m"], output("x86_64\n", 0))
            .with_env("XDG_SESSION_TYPE", "wayland")
            .with_env("XDG_CURRENT_DESKTOP", "GNOME")
            .with_file(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
                "performance\n",
            )
            .with_command(
                "lspci",
                &["-mm"],
                output(
                    "00:02.0 \"VGA compatible controller\" \"AMD\" \"RX 6800\"",
                    0,
                ),
            )
            .with_command("vulkaninfo", &["--summary"], output("", 1))
            .with_command(
                "glxinfo",
                &["-B"],
                output(
                    "OpenGL renderer string: AMD Radeon RX 6800\nOpenGL version string: 4.6 Mesa",
                    0,
                ),
            );

        let report = build_report(&fixture);
        let rendered = render::json::render_report(&report).unwrap();
        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(value["schema"], json!("gamedoc.v2"));
        assert_eq!(value["facts"].as_array().unwrap().len(), 11);
        assert_eq!(value["problems"][0]["id"], json!("gpu.vulkan_unavailable"));
        assert_eq!(value["problems"][0]["severity"], json!("critical"));
        assert!(
            value["checks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["operation"] == "vulkaninfo --summary"),
            "expected a diagnostic for vulkaninfo --summary"
        );
    }

    fn output(stdout: &str, exit_code: i32) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_owned(),
            stderr: String::new(),
            exit_code: Some(exit_code),
        }
    }
}
