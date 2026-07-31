use crate::collect::{diagnostic, CollectionResult, Collector};
use crate::facts::{Category, Fact, FactSource, ObservationMethod};
use crate::ids::{collector, fact, CollectorId};
use crate::probe::SystemProbe;

pub struct CpuCollector;

impl Collector for CpuCollector {
    fn id(&self) -> CollectorId {
        collector::CPU
    }

    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult {
        let mut result = CollectionResult::default();

        match system.read(std::path::Path::new("/proc/cpuinfo")) {
            Ok(content) => {
                for line in content.lines() {
                    let Some((key, value)) = line.split_once(':') else {
                        continue;
                    };
                    let key = key.trim();
                    let value = value.trim();
                    match key {
                        "vendor_id" => {
                            result.facts.push(observed(
                                fact::CPU_VENDOR,
                                value,
                                ObservationMethod::File("/proc/cpuinfo".to_owned()),
                            ));
                        }
                        "model name" => {
                            result.facts.push(observed(
                                fact::CPU_MODEL,
                                value,
                                ObservationMethod::File("/proc/cpuinfo".to_owned()),
                            ));
                        }
                        "cpu MHz" => {
                            result.facts.push(observed(
                                fact::CPU_FREQUENCY_MHZ,
                                value,
                                ObservationMethod::File("/proc/cpuinfo".to_owned()),
                            ));
                        }
                        "flags" => {
                            let flags: Vec<&str> = value
                                .split_whitespace()
                                .filter(|f| is_virtualization_flag(f))
                                .collect();
                            if !flags.is_empty() {
                                result.facts.push(observed(
                                    fact::CPU_VIRTUALIZATION_FLAGS,
                                    flags,
                                    ObservationMethod::File("/proc/cpuinfo".to_owned()),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(error) => {
                result
                    .diagnostics
                    .push(diagnostic(self.id(), "read /proc/cpuinfo", error));
            }
        }

        let logical_cores = count_logical_cores(system);
        result.facts.push(observed(
            fact::CPU_CORES_LOGICAL,
            logical_cores,
            ObservationMethod::Path("/sys/devices/system/cpu/".to_owned()),
        ));

        let physical_cores = count_physical_cores(system);
        if physical_cores > 0 {
            result.facts.push(observed(
                fact::CPU_CORES_PHYSICAL,
                physical_cores,
                ObservationMethod::Path("/sys/devices/system/cpu/".to_owned()),
            ));
        }

        if let Ok(output) = system.command("uname", &["-m"]) {
            let arch = output.stdout.trim();
            if !arch.is_empty() {
                result.facts.push(observed(
                    fact::CPU_ARCHITECTURE,
                    arch,
                    ObservationMethod::Command("uname -m".to_owned()),
                ));
            }
        }

        result
    }
}

fn count_logical_cores(system: &dyn SystemProbe) -> usize {
    match system.read_dir_names(std::path::Path::new("/sys/devices/system/cpu")) {
        Ok(names) => names
            .into_iter()
            .filter(|name| name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()))
            .count(),
        Err(_) => 0,
    }
}

fn count_physical_cores(system: &dyn SystemProbe) -> usize {
    let cpu_dir = std::path::Path::new("/sys/devices/system/cpu");
    let Ok(names) = system.read_dir_names(cpu_dir) else {
        return 0;
    };

    let mut unique = std::collections::BTreeSet::new();
    for name in names {
        if !name.starts_with("cpu") || !name[3..].chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let sibling_file = cpu_dir.join(&name).join("topology/core_siblings_list");
        let Ok(content) = system.read(&sibling_file) else {
            continue;
        };
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_owned());
        }
    }
    if unique.is_empty() {
        0
    } else {
        unique.len()
    }
}

fn observed(
    id: crate::ids::FactId,
    value: impl Into<serde_json::Value>,
    method: ObservationMethod,
) -> Fact {
    Fact::observed(
        id,
        Category::Cpu,
        value,
        FactSource {
            collector: collector::CPU,
            method,
        },
    )
}

fn is_virtualization_flag(flag: &str) -> bool {
    matches!(flag, "vmx" | "svm" | "hypervisor" | "svm_lock")
}

#[cfg(test)]
mod tests {
    use crate::collect::Collector;
    use crate::ids::fact;
    use crate::probe::FixtureSystemProbe;

    use super::CpuCollector;

    #[test]
    fn cpuinfo_and_core_counts_are_observed() {
        let cpuinfo = r"processor       : 0
vendor_id       : GenuineIntel
cpu family      : 6
model           : 151
model name      : 12th Gen Intel(R) Core(TM) i7-12700H
cpu MHz         : 2700.000

cache size      : 24576 KB

processor       : 1
vendor_id       : GenuineIntel
cpu family      : 6
model name      : 12th Gen Intel(R) Core(TM) i7-12700H
cpu MHz         : 2700.000
";
        let fixture = FixtureSystemProbe::new()
            .with_file("/proc/cpuinfo", cpuinfo)
            .with_command(
                "uname",
                &["-m"],
                crate::collect::command::CommandOutput {
                    stdout: "x86_64\n".to_owned(),
                    stderr: String::new(),
                    exit_code: Some(0),
                },
            )
            .with_directory(
                "/sys/devices/system/cpu",
                vec!["cpu0".to_owned(), "cpu1".to_owned(), "cpufreq".to_owned()],
            )
            .with_file(
                "/sys/devices/system/cpu/cpu0/topology/core_siblings_list",
                "0-3",
            )
            .with_file(
                "/sys/devices/system/cpu/cpu1/topology/core_siblings_list",
                "0-3",
            );

        let result = CpuCollector.collect(&fixture);
        assert!(result.facts.iter().any(|f| f.id == fact::CPU_VENDOR));
        assert!(result.facts.iter().any(|f| f.id == fact::CPU_MODEL));
        assert!(result.facts.iter().any(|f| f.id == fact::CPU_FREQUENCY_MHZ));
        assert!(result.facts.iter().any(|f| f.id == fact::CPU_ARCHITECTURE));
        assert_eq!(
            result
                .facts
                .iter()
                .find(|f| f.id == fact::CPU_ARCHITECTURE)
                .unwrap()
                .value,
            "x86_64"
        );
        assert_eq!(
            result
                .facts
                .iter()
                .find(|f| f.id == fact::CPU_CORES_LOGICAL)
                .unwrap()
                .value,
            2
        );
        assert_eq!(
            result
                .facts
                .iter()
                .find(|f| f.id == fact::CPU_CORES_PHYSICAL)
                .unwrap()
                .value,
            1
        );
    }

    #[test]
    fn unreadable_cpuinfo_is_diagnostic() {
        let fixture = FixtureSystemProbe::new();

        let result = CpuCollector.collect(&fixture);
        assert!(!result.diagnostics.is_empty());
    }
}
