use std::path::{Path, PathBuf};

use crate::collect::{CollectionResult, Collector};
use crate::facts::{Category, Fact, FactSource, ObservationMethod};
use crate::ids::{collector, fact, CollectorId};
use crate::probe::SystemProbe;

pub struct MangoHudCollector;

impl Collector for MangoHudCollector {
    fn id(&self) -> CollectorId {
        collector::MANGOHUD
    }

    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult {
        let mut result = CollectionResult::default();

        if system.which("mangohud").is_some() {
            result.facts.push(observed(
                fact::MANGOHUD_EXECUTABLE,
                true,
                ObservationMethod::Path("PATH:mangohud".to_owned()),
            ));
            result.facts.push(observed(
                fact::MANGOHUD_INSTALLATION,
                "executable",
                ObservationMethod::Path("MangoHud executable".to_owned()),
            ));
        }

        if let Some(home) = system.env("HOME").map(PathBuf::from) {
            collect_config(
                system,
                &home.join(".config/MangoHud/MangoHud.conf"),
                "user",
                &mut result,
            );
        }
        collect_config(
            system,
            Path::new("/etc/MangoHud.conf"),
            "system",
            &mut result,
        );

        for (name, evidence) in [
            ("MANGOHUD", "enable flag"),
            ("MANGOHUD_CONFIG", "inline configuration"),
            ("MANGOHUD_CONFIGFILE", "configuration file override"),
        ] {
            if system.env(name).is_some() {
                result.facts.push(observed(
                    fact::MANGOHUD_ENVIRONMENT,
                    evidence,
                    ObservationMethod::Environment("MangoHud setting".to_owned()),
                ));
            }
        }

        result
    }
}

fn collect_config(
    system: &dyn SystemProbe,
    path: &Path,
    scope: &str,
    result: &mut CollectionResult,
) {
    if system.exists(path) {
        result.facts.push(observed(
            fact::MANGOHUD_CONFIG,
            scope,
            ObservationMethod::Path(format!("{scope} MangoHud config")),
        ));
    }
}

fn observed(
    id: crate::ids::FactId,
    value: impl Into<serde_json::Value>,
    method: ObservationMethod,
) -> Fact {
    Fact::observed(
        id,
        Category::MangoHud,
        value,
        FactSource {
            collector: collector::MANGOHUD,
            method,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::collect::Collector;
    use crate::ids::fact;
    use crate::probe::FixtureSystemProbe;

    use super::MangoHudCollector;

    #[test]
    fn installation_config_and_environment_are_independent_observations() {
        let fixture = FixtureSystemProbe::new()
            .with_env("HOME", "/home/tester")
            .with_env("MANGOHUD", "1")
            .with_executable("mangohud", "/usr/bin/mangohud")
            .with_file("/home/tester/.config/MangoHud/MangoHud.conf", "fps");

        let result = MangoHudCollector.collect(&fixture);

        for id in [
            fact::MANGOHUD_EXECUTABLE,
            fact::MANGOHUD_INSTALLATION,
            fact::MANGOHUD_CONFIG,
            fact::MANGOHUD_ENVIRONMENT,
        ] {
            assert!(result.facts.iter().any(|item| item.id == id));
        }
        assert!(result.diagnostics.is_empty());
    }
}
