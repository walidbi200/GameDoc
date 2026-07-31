use std::path::Path;

use crate::collect::{diagnostic, CollectionResult, Collector};
use crate::facts::{Category, Fact, FactSource, ObservationMethod};
use crate::ids::{collector, fact, CollectorId};
use crate::probe::SystemProbe;

pub struct SystemCollector;

impl Collector for SystemCollector {
    fn id(&self) -> CollectorId {
        collector::SYSTEM
    }

    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult {
        let mut result = CollectionResult::default();

        collect_command_fact(system, &mut result, fact::KERNEL_VERSION, "uname", &["-r"]);
        collect_command_fact(system, &mut result, fact::ARCHITECTURE, "uname", &["-m"]);
        collect_env_fact(
            system,
            &mut result,
            fact::SESSION_TYPE,
            "XDG_SESSION_TYPE",
            "desktop session type",
        );
        collect_env_fact(
            system,
            &mut result,
            fact::DESKTOP,
            "XDG_CURRENT_DESKTOP",
            "desktop environment",
        );

        let governor = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor");
        match system.read(governor) {
            Ok(value) if !value.trim().is_empty() => result.facts.push(Fact::observed(
                fact::CPU_GOVERNOR,
                Category::System,
                value.trim(),
                FactSource {
                    collector: self.id(),
                    method: ObservationMethod::File(governor.display().to_string()),
                },
            )),
            Ok(_) => {}
            Err(error) => result.diagnostics.push(diagnostic(
                self.id(),
                format!("read {}", governor.display()),
                error,
            )),
        }

        result
    }
}

fn collect_command_fact(
    system: &dyn SystemProbe,
    result: &mut CollectionResult,
    id: crate::ids::FactId,
    program: &str,
    args: &[&str],
) {
    match system.command(program, args) {
        Ok(output) if output.success() && !output.stdout.trim().is_empty() => {
            result.facts.push(Fact::observed(
                id,
                Category::System,
                output.stdout.trim(),
                FactSource {
                    collector: collector::SYSTEM,
                    method: ObservationMethod::Command(format_command(program, args)),
                },
            ));
        }
        Ok(output) => result.diagnostics.push(diagnostic(
            collector::SYSTEM,
            format_command(program, args),
            format!("command exited with status {:?}", output.exit_code),
        )),
        Err(error) => result.diagnostics.push(diagnostic(
            collector::SYSTEM,
            format_command(program, args),
            error,
        )),
    }
}

fn collect_env_fact(
    system: &dyn SystemProbe,
    result: &mut CollectionResult,
    id: crate::ids::FactId,
    name: &str,
    public_label: &str,
) {
    if let Some(value) = system.env(name).and_then(|value| value.into_string().ok()) {
        if !value.is_empty() {
            result.facts.push(Fact::observed(
                id,
                Category::System,
                value,
                FactSource {
                    collector: collector::SYSTEM,
                    method: ObservationMethod::Environment(public_label.to_owned()),
                },
            ));
        }
    }
}

fn format_command(program: &str, args: &[&str]) -> String {
    std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use crate::collect::command::CommandOutput;
    use crate::collect::Collector;
    use crate::ids::fact;
    use crate::probe::FixtureSystemProbe;

    use super::SystemCollector;

    #[test]
    fn fixture_collection_emits_observations_only() {
        let fixture = FixtureSystemProbe::new()
            .with_command("uname", &["-r"], output("6.9.0"))
            .with_command("uname", &["-m"], output("x86_64"))
            .with_env("XDG_SESSION_TYPE", "wayland")
            .with_file(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor",
                "performance\n",
            );

        let result = SystemCollector.collect(&fixture);

        assert!(result
            .facts
            .iter()
            .any(|item| item.id == fact::KERNEL_VERSION));
        assert!(result
            .facts
            .iter()
            .any(|item| item.id == fact::CPU_GOVERNOR));
        assert!(result.diagnostics.is_empty());
    }

    fn output(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: format!("{stdout}\n"),
            stderr: String::new(),
            exit_code: Some(0),
        }
    }
}
