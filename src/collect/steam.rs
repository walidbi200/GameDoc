use std::path::{Path, PathBuf};

use serde_json::json;

use crate::collect::{diagnostic, CollectionResult, Collector};
use crate::facts::{Category, Fact, FactSource, ObservationMethod};
use crate::ids::{collector, fact, CollectorId};
use crate::probe::SystemProbe;

pub struct SteamCollector;

struct Installation {
    kind: &'static str,
    root: PathBuf,
}

impl Collector for SteamCollector {
    fn id(&self) -> CollectorId {
        collector::STEAM
    }

    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult {
        let mut result = CollectionResult::default();
        let home = system.env("HOME").map(PathBuf::from);

        if system.which("steam").is_some() {
            result.facts.push(observed(
                fact::STEAM_EXECUTABLE,
                true,
                ObservationMethod::Path("PATH:steam".to_owned()),
            ));
            result.facts.push(observed(
                fact::STEAM_INSTALLATION,
                "native",
                ObservationMethod::Path("native Steam executable".to_owned()),
            ));
        }

        let installations = installations(home.as_deref());
        for installation in installations
            .iter()
            .filter(|installation| system.exists(&installation.root))
        {
            collect_installation(system, installation, &mut result);
        }

        if result
            .facts
            .iter()
            .any(|item| item.id == fact::STEAM_EXECUTABLE)
            && !result
                .facts
                .iter()
                .any(|item| item.id == fact::STEAM_INSTALLATION)
        {
            result.diagnostics.push(diagnostic(
                self.id(),
                "steam data",
                "Steam executable was found but no supported data directory was readable",
            ));
        }

        result
    }
}

fn installations(home: Option<&Path>) -> Vec<Installation> {
    let Some(home) = home else {
        return Vec::new();
    };
    vec![
        Installation {
            kind: "native",
            root: home.join(".local/share/Steam"),
        },
        Installation {
            kind: "flatpak",
            root: home.join(".var/app/com.valvesoftware.Steam/data/Steam"),
        },
        Installation {
            kind: "snap",
            root: home.join("snap/steam/common/.local/share/Steam"),
        },
    ]
}

fn collect_installation(
    system: &dyn SystemProbe,
    installation: &Installation,
    result: &mut CollectionResult,
) {
    if !result.facts.iter().any(|item| {
        item.id == fact::STEAM_INSTALLATION && item.value.as_str() == Some(installation.kind)
    }) {
        result.facts.push(observed(
            fact::STEAM_INSTALLATION,
            installation.kind,
            ObservationMethod::Path(format!("{} Steam data", installation.kind)),
        ));
    }

    let steamapps = installation.root.join("steamapps");
    if system.read_dir_names(&steamapps).is_ok() {
        result.facts.push(observed(
            fact::STEAM_DATA_READABLE,
            json!({"installation": installation.kind, "readable": true}),
            ObservationMethod::Path(format!("{} Steam data", installation.kind)),
        ));
        collect_libraries(system, installation, &steamapps, result);
    } else {
        result.facts.push(observed(
            fact::STEAM_DATA_READABLE,
            json!({"installation": installation.kind, "readable": false}),
            ObservationMethod::Path(format!("{} Steam data", installation.kind)),
        ));
        result.diagnostics.push(diagnostic(
            collector::STEAM,
            format!("{} Steam data", installation.kind),
            "Steam data directory could not be read",
        ));
    }

    collect_tools(
        system,
        installation,
        &installation.root.join("compatibilitytools.d"),
        result,
    );
}

fn collect_libraries(
    system: &dyn SystemProbe,
    installation: &Installation,
    primary_steamapps: &Path,
    result: &mut CollectionResult,
) {
    let mut libraries = vec![primary_steamapps.to_owned()];
    let metadata = primary_steamapps.join("libraryfolders.vdf");
    if let Ok(content) = system.read(&metadata) {
        for path in parse_library_paths(&content) {
            let steamapps = path.join("steamapps");
            if !libraries.contains(&steamapps) {
                libraries.push(steamapps);
            }
        }
    }

    for (index, steamapps) in libraries.iter().enumerate() {
        let label = json!({"installation": installation.kind, "index": index + 1});
        result.facts.push(observed(
            fact::STEAM_LIBRARY,
            label.clone(),
            ObservationMethod::File(format!("{} Steam library metadata", installation.kind)),
        ));
        if system.read_dir_names(steamapps).is_ok() {
            result.facts.push(observed(
                fact::STEAM_LIBRARY_READABLE,
                json!({"installation": installation.kind, "index": index + 1, "readable": true}),
                ObservationMethod::Path("Steam library".to_owned()),
            ));
            collect_proton(system, installation.kind, &steamapps.join("common"), result);
        } else {
            result.facts.push(observed(
                fact::STEAM_LIBRARY_READABLE,
                json!({"installation": installation.kind, "index": index + 1, "readable": false}),
                ObservationMethod::Path("Steam library".to_owned()),
            ));
            result.diagnostics.push(diagnostic(
                collector::STEAM,
                format!("{} Steam library {}", installation.kind, index + 1),
                "Steam library could not be read",
            ));
        }
    }
}

fn collect_proton(
    system: &dyn SystemProbe,
    installation: &str,
    common: &Path,
    result: &mut CollectionResult,
) {
    if let Ok(names) = system.read_dir_names(common) {
        result.facts.push(observed(
            fact::STEAM_PROTON_SCAN_COMPLETE,
            json!({"installation": installation, "complete": true}),
            ObservationMethod::Path("Steam common directory".to_owned()),
        ));
        for name in names.into_iter().filter(|name| is_proton_name(name)) {
            result.facts.push(observed(
                fact::STEAM_PROTON_VERSION,
                json!({"installation": installation, "name": name}),
                ObservationMethod::Path("Steam common directory".to_owned()),
            ));
        }
    } else {
        result.facts.push(observed(
            fact::STEAM_PROTON_SCAN_COMPLETE,
            json!({"installation": installation, "complete": false}),
            ObservationMethod::Path("Steam common directory".to_owned()),
        ));
        result.diagnostics.push(diagnostic(
            collector::STEAM,
            format!("{installation} Steam common directory"),
            "Steam common directory could not be read",
        ));
    }
}

fn collect_tools(
    system: &dyn SystemProbe,
    installation: &Installation,
    directory: &Path,
    result: &mut CollectionResult,
) {
    if !system.exists(directory) {
        return;
    }
    match system.read_dir_names(directory) {
        Ok(names) => {
            for name in names {
                result.facts.push(observed(
                    fact::STEAM_COMPATIBILITY_TOOL,
                    json!({"installation": installation.kind, "name": name}),
                    ObservationMethod::Path("Steam compatibility tools".to_owned()),
                ));
            }
        }
        Err(_) => result.diagnostics.push(diagnostic(
            collector::STEAM,
            format!("{} compatibility tools", installation.kind),
            "Compatibility tool directory could not be read",
        )),
    }
}

fn parse_library_paths(content: &str) -> Vec<PathBuf> {
    content
        .lines()
        .filter_map(|line| {
            let fields = quoted_fields(line);
            (fields.first().map(String::as_str) == Some("path"))
                .then(|| fields.get(1).map(PathBuf::from))
                .flatten()
        })
        .collect()
}

fn quoted_fields(line: &str) -> Vec<String> {
    line.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

fn is_proton_name(name: &str) -> bool {
    let lowercase = name.to_ascii_lowercase();
    lowercase.starts_with("proton ") || lowercase.starts_with("proton-")
}

fn observed(
    id: crate::ids::FactId,
    value: impl Into<serde_json::Value>,
    method: ObservationMethod,
) -> Fact {
    Fact::observed(
        id,
        Category::Steam,
        value,
        FactSource {
            collector: collector::STEAM,
            method,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::collect::Collector;
    use crate::ids::fact;
    use crate::probe::FixtureSystemProbe;

    use super::SteamCollector;

    #[test]
    fn native_multiple_libraries_and_proton_are_observed() {
        let fixture = FixtureSystemProbe::new()
            .with_env("HOME", "/home/tester")
            .with_executable("steam", "/usr/bin/steam")
            .with_directory(
                "/home/tester/.local/share/Steam",
                vec!["steamapps".to_owned()],
            )
            .with_directory(
                "/home/tester/.local/share/Steam/steamapps",
                vec!["libraryfolders.vdf".to_owned()],
            )
            .with_file(
                "/home/tester/.local/share/Steam/steamapps/libraryfolders.vdf",
                "\"path\" \"/games/SteamLibrary\"",
            )
            .with_directory(
                "/home/tester/.local/share/Steam/steamapps/common",
                vec!["Proton Experimental".to_owned()],
            )
            .with_directory("/games/SteamLibrary/steamapps", Vec::new())
            .with_directory(
                "/games/SteamLibrary/steamapps/common",
                vec!["Proton Hotfix".to_owned()],
            );

        let result = SteamCollector.collect(&fixture);

        assert_eq!(
            result
                .facts
                .iter()
                .filter(|item| item.id == fact::STEAM_LIBRARY)
                .count(),
            2
        );
        assert_eq!(
            result
                .facts
                .iter()
                .filter(|item| item.id == fact::STEAM_PROTON_VERSION)
                .count(),
            2
        );
    }

    #[test]
    fn unreadable_library_is_a_fact_and_diagnostic() {
        let fixture = FixtureSystemProbe::new()
            .with_env("HOME", "/home/tester")
            .with_directory(
                "/home/tester/.local/share/Steam",
                vec!["steamapps".to_owned()],
            )
            .with_directory(
                "/home/tester/.local/share/Steam/steamapps",
                vec!["libraryfolders.vdf".to_owned()],
            )
            .with_file(
                "/home/tester/.local/share/Steam/steamapps/libraryfolders.vdf",
                "\"path\" \"/games/private\"",
            )
            .with_unreadable_directory("/games/private/steamapps");

        let result = SteamCollector.collect(&fixture);

        assert!(result.facts.iter().any(|item| {
            item.id == fact::STEAM_LIBRARY_READABLE
                && item.value["readable"] == serde_json::Value::Bool(false)
        }));
        assert_eq!(result.diagnostics.len(), 2);
    }
}
