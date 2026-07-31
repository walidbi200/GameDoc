use crate::collect::{diagnostic, CollectionResult, Collector};
use crate::facts::{Category, Fact, FactSource, ObservationMethod};
use crate::ids::{collector, fact, CollectorId};
use crate::probe::SystemProbe;

pub struct DistroCollector;

impl Collector for DistroCollector {
    fn id(&self) -> CollectorId {
        collector::DISTRO
    }

    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult {
        let mut result = CollectionResult::default();
        let path = std::path::Path::new("/etc/os-release");

        match system.read(path) {
            Ok(content) => {
                let fields = parse_os_release(&content);
                if let Some(name) = fields.get("NAME") {
                    result.facts.push(observed(
                        fact::DISTRO_NAME,
                        name.as_str(),
                        ObservationMethod::File("/etc/os-release".to_owned()),
                    ));
                }
                if let Some(id) = fields.get("ID") {
                    result.facts.push(observed(
                        fact::DISTRO_ID,
                        id.as_str(),
                        ObservationMethod::File("/etc/os-release".to_owned()),
                    ));
                }
                if let Some(version) = fields.get("VERSION_ID") {
                    result.facts.push(observed(
                        fact::DISTRO_VERSION_ID,
                        version.as_str(),
                        ObservationMethod::File("/etc/os-release".to_owned()),
                    ));
                }
                if let Some(pretty) = fields.get("PRETTY_NAME") {
                    result.facts.push(observed(
                        fact::DISTRO_PRETTY_NAME,
                        pretty.as_str(),
                        ObservationMethod::File("/etc/os-release".to_owned()),
                    ));
                }
                let family = derive_family(&fields);
                result.facts.push(observed(
                    fact::DISTRO_FAMILY,
                    family,
                    ObservationMethod::File("/etc/os-release".to_owned()),
                ));
            }
            Err(error) => {
                result
                    .diagnostics
                    .push(diagnostic(self.id(), "read /etc/os-release", error));
            }
        }

        result
    }
}

fn parse_os_release(content: &str) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, raw_value)) = trimmed.split_once('=') {
            let value = raw_value.trim().trim_matches('"').to_owned();
            map.insert(key.to_owned(), value);
        }
    }
    map
}

fn derive_family(fields: &std::collections::BTreeMap<String, String>) -> &'static str {
    let id = fields.get("ID").map(String::as_str).unwrap_or_default();
    let id_like = fields
        .get("ID_LIKE")
        .map(String::as_str)
        .unwrap_or_default();

    match id {
        "fedora" => "fedora",
        "arch" | "manjaro" => "arch",
        "ubuntu" | "debian" => "debian",
        "opensuse" | "suse" => "suse",
        "nixos" => "nixos",
        "bazzite" => "bazzite",
        "steamos" => "steamos",
        _ => {
            if id_like.contains("fedora") {
                "fedora"
            } else if id_like.contains("arch") {
                "arch"
            } else if id_like.contains("debian") {
                "debian"
            } else if id_like.contains("suse") {
                "suse"
            } else if id == "unknown" || id.is_empty() {
                "unknown"
            } else {
                "universal"
            }
        }
    }
}

fn observed(
    id: crate::ids::FactId,
    value: impl Into<serde_json::Value>,
    method: ObservationMethod,
) -> Fact {
    Fact::observed(
        id,
        Category::Distro,
        value,
        FactSource {
            collector: collector::DISTRO,
            method,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::collect::Collector;
    use crate::ids::fact;
    use crate::probe::FixtureSystemProbe;

    use super::DistroCollector;

    #[test]
    fn fedora_parsing() {
        let content = r#"NAME="Fedora Linux"
VERSION="40 (Workstation Edition)"
ID=fedora
VERSION_ID=40
PRETTY_NAME="Fedora Linux 40 (Workstation Edition)"
ID_LIKE="rhel centos fedora"
"#;
        let fixture = FixtureSystemProbe::new().with_file("/etc/os-release", content);

        let result = DistroCollector.collect(&fixture);
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_NAME && f.value == "Fedora Linux"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_ID && f.value == "fedora"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_VERSION_ID && f.value == "40"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_FAMILY && f.value == "fedora"));
    }

    #[test]
    fn arch_parsing() {
        let content = r#"NAME="Arch Linux"
ID=arch
PRETTY_NAME="Arch Linux"
ID_LIKE="arch"
"#;
        let fixture = FixtureSystemProbe::new().with_file("/etc/os-release", content);

        let result = DistroCollector.collect(&fixture);
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_ID && f.value == "arch"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_FAMILY && f.value == "arch"));
    }

    #[test]
    fn ubuntu_parsing() {
        let content = r#"NAME="Ubuntu"
VERSION_ID="24.04"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 24.04 LTS"
"#;
        let fixture = FixtureSystemProbe::new().with_file("/etc/os-release", content);

        let result = DistroCollector.collect(&fixture);
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_ID && f.value == "ubuntu"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_VERSION_ID && f.value == "24.04"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_FAMILY && f.value == "debian"));
    }

    #[test]
    fn bazzite_parsing() {
        let content = r#"NAME="Bazzite"
ID=bazzite
PRETTY_NAME="Bazzite"
"#;
        let fixture = FixtureSystemProbe::new().with_file("/etc/os-release", content);

        let result = DistroCollector.collect(&fixture);
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_ID && f.value == "bazzite"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_FAMILY && f.value == "bazzite"));
    }

    #[test]
    fn steamos_parsing() {
        let content = r#"NAME="SteamOS"
ID=steamos
PRETTY_NAME="SteamOS Holo"
"#;
        let fixture = FixtureSystemProbe::new().with_file("/etc/os-release", content);

        let result = DistroCollector.collect(&fixture);
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_ID && f.value == "steamos"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_FAMILY && f.value == "steamos"));
    }

    #[test]
    fn nixos_parsing() {
        let content = r#"NAME="NixOS"
ID=nixos
PRETTY_NAME="NixOS 24.05"
"#;
        let fixture = FixtureSystemProbe::new().with_file("/etc/os-release", content);

        let result = DistroCollector.collect(&fixture);
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_ID && f.value == "nixos"));
        assert!(result
            .facts
            .iter()
            .any(|f| f.id == fact::DISTRO_FAMILY && f.value == "nixos"));
    }

    #[test]
    fn missing_os_release() {
        let fixture = FixtureSystemProbe::new();

        let result = DistroCollector.collect(&fixture);
        assert!(result.facts.is_empty());
        assert!(!result.diagnostics.is_empty());
    }
}
