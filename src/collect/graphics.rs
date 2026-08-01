use crate::collect::{diagnostic, CollectionResult, Collector};
use crate::facts::{Category, Fact, FactSource, ObservationMethod};
use crate::ids::{collector, fact, CollectorId};
use crate::probe::SystemProbe;

pub struct GraphicsCollector;

impl Collector for GraphicsCollector {
    fn id(&self) -> CollectorId {
        collector::GRAPHICS
    }

    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult {
        let mut result = CollectionResult::default();

        collect_gpu_devices(system, &mut result);
        collect_vulkan(system, &mut result);
        collect_opengl(system, &mut result);

        result
    }
}

fn collect_gpu_devices(system: &dyn SystemProbe, result: &mut CollectionResult) {
    match system.command("lspci", &["-mm"]) {
        Ok(output) if output.success() => {
            for line in output.stdout.lines().filter(|line| is_display_device(line)) {
                let parsed = clean_gpu_name(&parse_lspci_mm_line(line));
                result.facts.push(Fact::observed(
                    fact::GPU_DEVICE,
                    Category::Gpu,
                    parsed,
                    command_source("lspci -mm"),
                ));
            }
        }
        Ok(output) => result.diagnostics.push(diagnostic(
            collector::GRAPHICS,
            "lspci -mm",
            format!("command exited with status {:?}", output.exit_code),
        )),
        Err(error) => result
            .diagnostics
            .push(diagnostic(collector::GRAPHICS, "lspci -mm", error)),
    }
}

fn collect_vulkan(system: &dyn SystemProbe, result: &mut CollectionResult) {
    match system.command("vulkaninfo", &["--summary"]) {
        Ok(output) => {
            result.facts.push(Fact::observed(
                fact::VULKAN_AVAILABLE,
                Category::Gpu,
                output.success(),
                command_source("vulkaninfo --summary"),
            ));
            if !output.success() {
                result.diagnostics.push(diagnostic(
                    collector::GRAPHICS,
                    "vulkaninfo --summary",
                    format!("command exited with status {:?}", output.exit_code),
                ));
            }
        }
        Err(error) => result.diagnostics.push(diagnostic(
            collector::GRAPHICS,
            "vulkaninfo --summary",
            error,
        )),
    }
}

fn collect_opengl(system: &dyn SystemProbe, result: &mut CollectionResult) {
    match system.command("glxinfo", &["-B"]) {
        Ok(output) if output.success() => {
            for line in output.stdout.lines() {
                let Some((key, value)) = line.split_once(':') else {
                    continue;
                };
                let id = match key.trim() {
                    "OpenGL renderer string" => fact::OPENGL_RENDERER,
                    "OpenGL version string" => fact::OPENGL_VERSION,
                    _ => continue,
                };
                result.facts.push(Fact::observed(
                    id,
                    Category::Gpu,
                    value.trim(),
                    command_source("glxinfo -B"),
                ));
            }
        }
        Ok(output) => result.diagnostics.push(diagnostic(
            collector::GRAPHICS,
            "glxinfo -B",
            format!("command exited with status {:?}", output.exit_code),
        )),
        Err(error) => result
            .diagnostics
            .push(diagnostic(collector::GRAPHICS, "glxinfo -B", error)),
    }
}

fn command_source(command: &str) -> FactSource {
    FactSource {
        collector: collector::GRAPHICS,
        method: ObservationMethod::Command(command.to_owned()),
    }
}

fn is_display_device(line: &str) -> bool {
    line.contains("VGA compatible controller")
        || line.contains("3D controller")
        || line.contains("Display controller")
}

fn parse_lspci_mm_line(line: &str) -> String {
    let parts: Vec<&str> = line.split("\" \"").collect();
    if parts.len() >= 3 {
        let vendor = parts[1].trim_matches('"');
        let device = parts[2].trim_matches('"');
        format!("{vendor} {device}")
    } else {
        line.trim().to_owned()
    }
}

fn clean_gpu_name(raw: &str) -> String {
    raw.replace('\"', "")
        .replace(" Corporation", "")
        .replace(" Micro Devices, Inc.", "")
        .replace(" [AMD/ATI]", "")
        .replace(" (R)", "")
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use crate::collect::command::CommandOutput;
    use crate::collect::Collector;
    use crate::ids::fact;
    use crate::probe::FixtureSystemProbe;

    use super::GraphicsCollector;

    #[test]
    fn repeated_gpu_devices_are_preserved() {
        let fixture = FixtureSystemProbe::new()
            .with_command(
                "lspci",
                &["-mm"],
                output(
                    "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"UHD\"\n03:00.0 \"3D controller\" \"AMD\" \"RX 6800\"",
                ),
            )
            .with_command("vulkaninfo", &["--summary"], output("Vulkan Instance Version"))
            .with_command(
                "glxinfo",
                &["-B"],
                output("OpenGL renderer string: AMD Radeon RX 6800\nOpenGL version string: 4.6 Mesa"),
            );

        let result = GraphicsCollector.collect(&fixture);

        assert_eq!(
            result
                .facts
                .iter()
                .filter(|item| item.id == fact::GPU_DEVICE)
                .count(),
            2
        );
        let gpus: Vec<_> = result
            .facts
            .iter()
            .filter(|f| f.id == fact::GPU_DEVICE)
            .collect();
        assert_eq!(gpus[0].value.as_str().unwrap(), "Intel UHD");
        assert_eq!(gpus[1].value.as_str().unwrap(), "AMD RX 6800");
        assert!(result
            .facts
            .iter()
            .any(|item| item.id == fact::VULKAN_AVAILABLE));
    }

    fn output(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_owned(),
            stderr: String::new(),
            exit_code: Some(0),
        }
    }
}
