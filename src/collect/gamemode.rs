use std::path::{Path, PathBuf};

use crate::collect::{diagnostic, CollectionResult, Collector};
use crate::facts::{Category, Fact, FactSource, ObservationMethod};
use crate::ids::{collector, fact, CollectorId};
use crate::probe::SystemProbe;

pub struct GameModeCollector;

impl Collector for GameModeCollector {
    fn id(&self) -> CollectorId {
        collector::GAMEMODE
    }

    fn collect(&self, system: &dyn SystemProbe) -> CollectionResult {
        let mut result = CollectionResult::default();

        if system.which("gamemoderun").is_some() {
            result.facts.push(observed(
                fact::GAMEMODE_EXECUTABLE,
                true,
                ObservationMethod::Path("PATH:gamemoderun".to_owned()),
            ));
        }

        if system.which("gamemoded").is_some() {
            match system.command("gamemoded", &["-s"]) {
                Ok(output) => {
                    result.facts.push(observed(
                        fact::GAMEMODE_DAEMON_AVAILABLE,
                        output.success(),
                        ObservationMethod::Command("gamemoded -s".to_owned()),
                    ));
                    if !output.success() {
                        result.diagnostics.push(diagnostic(
                            self.id(),
                            "gamemoded -s",
                            "GameMode daemon status probe failed",
                        ));
                    }
                }
                Err(_) => result.diagnostics.push(diagnostic(
                    self.id(),
                    "gamemoded -s",
                    "GameMode daemon status could not be queried",
                )),
            }
            if let Ok(output) = system.command("pgrep", &["gamemoded"]) {
                result.facts.push(observed(
                    fact::GAMEMODE_DAEMON_RUNNING,
                    output.success(),
                    ObservationMethod::Command("pgrep gamemoded".to_owned()),
                ));
            }
        } else if system.which("gamemoderun").is_some() {
            result.facts.push(observed(
                fact::GAMEMODE_DAEMON_AVAILABLE,
                false,
                ObservationMethod::Path("PATH:gamemoded".to_owned()),
            ));
        }

        for (path, architecture) in [
            ("/usr/lib64/libgamemode.so.0", "64-bit"),
            ("/usr/lib/libgamemode.so.0", "system"),
            ("/usr/lib32/libgamemode.so.0", "32-bit"),
        ] {
            if system.exists(Path::new(path)) {
                result.facts.push(observed(
                    fact::GAMEMODE_LIBRARY,
                    architecture,
                    ObservationMethod::Path(format!("{architecture} GameMode library")),
                ));
            }
        }

        collect_config(
            system,
            Path::new("/etc/gamemode.ini"),
            "system",
            &mut result,
        );
        if let Some(home) = system.env("HOME").map(PathBuf::from) {
            collect_config(
                system,
                &home.join(".config/gamemode.ini"),
                "user",
                &mut result,
            );
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
            fact::GAMEMODE_CONFIG,
            scope,
            ObservationMethod::Path(format!("{scope} GameMode config")),
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
        Category::GameMode,
        value,
        FactSource {
            collector: collector::GAMEMODE,
            method,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::collect::command::CommandOutput;
    use crate::collect::Collector;
    use crate::ids::fact;
    use crate::probe::FixtureSystemProbe;

    use super::GameModeCollector;

    #[test]
    fn executable_daemon_library_and_config_are_observed() {
        let fixture = FixtureSystemProbe::new()
            .with_env("HOME", "/home/tester")
            .with_executable("gamemoderun", "/usr/bin/gamemoderun")
            .with_executable("gamemoded", "/usr/bin/gamemoded")
            .with_command(
                "gamemoded",
                &["-s"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: Some(0),
                },
            )
            .with_file("/usr/lib64/libgamemode.so.0", "")
            .with_file("/home/tester/.config/gamemode.ini", "[general]");

        let result = GameModeCollector.collect(&fixture);

        for id in [
            fact::GAMEMODE_EXECUTABLE,
            fact::GAMEMODE_DAEMON_AVAILABLE,
            fact::GAMEMODE_LIBRARY,
            fact::GAMEMODE_CONFIG,
        ] {
            assert!(result.facts.iter().any(|item| item.id == id));
        }
    }
}
