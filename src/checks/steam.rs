use crate::collect::context::CollectorContext;
use crate::collect::{CollectionResult, Collector};
use crate::ids;
use crate::model::{Category, Evidence, EvidenceSource, Fact};
use std::path::Path;

pub struct SteamCollector;

impl Collector for SteamCollector {
    fn id(&self) -> &'static str {
        "steam"
    }

    fn collect(&self, ctx: &CollectorContext) -> CollectionResult {
        let mut res = CollectionResult::new();
        let home = std::env::var("HOME").unwrap_or_else(|_| String::new());

        // 1. Native Steam
        let native_installed = ctx.fs.exists(Path::new("/usr/bin/steam"))
            || ctx.fs.exists(Path::new("/usr/games/steam"));
        let native_root1 = format!("{home}/.local/share/Steam");
        let native_root2 = format!("{home}/.steam/steam");
        let native_root = if ctx.fs.exists(Path::new(&native_root1)) {
            Some(native_root1)
        } else if ctx.fs.exists(Path::new(&native_root2)) {
            Some(native_root2)
        } else {
            None
        };
        check_variant(
            ctx,
            &mut res,
            "Native",
            native_installed,
            native_root,
            Evidence::new(EvidenceSource::FileSystem, "Native paths"),
        );

        // 2. Flatpak Steam
        let flatpak_out = ctx
            .command
            .run("flatpak", &["info", "com.valvesoftware.Steam"]);
        let flatpak_installed = flatpak_out.is_ok() && flatpak_out.unwrap().success();
        let flatpak_root_path = format!("{home}/.var/app/com.valvesoftware.Steam/data/Steam");
        let flatpak_root = if ctx.fs.exists(Path::new(&flatpak_root_path)) {
            Some(flatpak_root_path)
        } else {
            None
        };
        check_variant(
            ctx,
            &mut res,
            "Flatpak",
            flatpak_installed,
            flatpak_root,
            Evidence::new(EvidenceSource::Command, "flatpak info"),
        );

        // 3. Snap Steam
        let snap_out = ctx.command.run("snap", &["list", "steam"]);
        let snap_installed = snap_out.is_ok() && snap_out.unwrap().success();
        let snap_root1 = format!("{home}/snap/steam/common/.local/share/Steam");
        let snap_root2 = format!("{home}/snap/steam/common/.steam/steam");
        let snap_root = if ctx.fs.exists(Path::new(&snap_root1)) {
            Some(snap_root1)
        } else if ctx.fs.exists(Path::new(&snap_root2)) {
            Some(snap_root2)
        } else {
            None
        };
        check_variant(
            ctx,
            &mut res,
            "Snap",
            snap_installed,
            snap_root,
            Evidence::new(EvidenceSource::Command, "snap list"),
        );

        res
    }
}

fn check_variant(
    ctx: &CollectorContext,
    res: &mut CollectionResult,
    variant_name: &str,
    is_installed: bool,
    initialized_root: Option<String>,
    evidence: Evidence,
) {
    if is_installed && initialized_root.is_none() {
        res.facts.push(Fact {
            id: ids::steam::INSTALLATION.to_owned(),
            category: Category::Steam,
            value: format!("{variant_name} (Uninitialized)"),
            evidence,
        });
    } else if let Some(root) = initialized_root {
        res.facts.push(Fact {
            id: ids::steam::INSTALLATION.to_owned(),
            category: Category::Steam,
            value: format!("{variant_name} (Initialized in {root})"),
            evidence: evidence.clone(),
        });

        find_proton(ctx, res, variant_name, &root, &evidence);
    }
}

fn find_proton(
    ctx: &CollectorContext,
    res: &mut CollectionResult,
    variant_name: &str,
    steam_root: &str,
    _evidence: &Evidence,
) {
    // Check official proton in steamapps/common
    let common_dir = format!("{steam_root}/steamapps/common");
    if let Ok(entries) = ctx.fs.read_dir_names(Path::new(&common_dir)) {
        for entry in entries {
            if entry.starts_with("Proton") {
                res.facts.push(Fact {
                    id: ids::steam::PROTON.to_owned(),
                    category: Category::Steam,
                    value: format!("{variant_name} Official: {entry}"),
                    evidence: Evidence::new(EvidenceSource::FileSystem, common_dir.clone()),
                });
            }
        }
    }

    // Check custom proton in compatibilitytools.d
    let custom_dir = format!("{steam_root}/compatibilitytools.d");
    if let Ok(entries) = ctx.fs.read_dir_names(Path::new(&custom_dir)) {
        for entry in entries {
            res.facts.push(Fact {
                id: ids::steam::COMPATIBILITY_TOOLS.to_owned(),
                category: Category::Steam,
                value: format!("{variant_name} Custom: {entry}"),
                evidence: Evidence::new(EvidenceSource::FileSystem, custom_dir.clone()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::SteamCollector;
    use crate::collect::command::{CommandOutput, CommandRunner};
    use crate::collect::context::CollectorContext;
    use crate::collect::filesystem::StubFilesystem;
    use crate::collect::Collector;
    use crate::errors::CollectError;
    use crate::ids;

    struct StubCommandRunner(HashMap<String, String>);

    impl CommandRunner for StubCommandRunner {
        #[allow(clippy::disallowed_methods)]
        fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CollectError> {
            let key = format!("{program} {}", args.join(" "));
            self.0.get(&key).map_or_else(
                || {
                    Err(CollectError::CommandNotFound {
                        program: program.to_owned(),
                    })
                },
                |stdout| {
                    Ok(CommandOutput {
                        stdout: stdout.clone(),
                        stderr: String::new(),
                        exit_code: Some(0),
                        duration_ms: 5,
                    })
                },
            )
        }
    }

    fn ctx(cmd: StubCommandRunner, fs: StubFilesystem) -> CollectorContext {
        std::env::set_var("HOME", "/home/test");
        CollectorContext::new(Arc::new(cmd), Arc::new(fs))
    }

    #[test]
    fn uninitialized_native_steam() {
        let fs = StubFilesystem::new().with_file("/usr/bin/steam", "");
        let res = SteamCollector.collect(&ctx(StubCommandRunner(HashMap::new()), fs));

        let fact = res
            .facts
            .iter()
            .find(|f| f.id == ids::steam::INSTALLATION)
            .unwrap();
        assert_eq!(fact.value, "Native (Uninitialized)");
    }

    #[test]
    fn initialized_flatpak_steam_with_custom_proton() {
        let mut cmds = HashMap::new();
        cmds.insert(
            "flatpak info com.valvesoftware.Steam".to_owned(),
            "installed".to_owned(),
        );

        let fs = StubFilesystem::new()
            .with_file("/home/test/.var/app/com.valvesoftware.Steam/data/Steam", "")
            .with_file("/home/test/.var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d/GE-Proton8-25", "");

        let res = SteamCollector.collect(&ctx(StubCommandRunner(cmds), fs));

        let inst = res
            .facts
            .iter()
            .find(|f| f.id == ids::steam::INSTALLATION)
            .unwrap();
        assert!(inst.value.contains("Flatpak (Initialized in"));

        let tool = res
            .facts
            .iter()
            .find(|f| f.id == ids::steam::COMPATIBILITY_TOOLS)
            .unwrap();
        assert_eq!(tool.value, "Flatpak Custom: GE-Proton8-25");
    }

    #[test]
    fn initialized_snap_steam_with_official_proton() {
        let mut cmds = HashMap::new();
        cmds.insert("snap list steam".to_owned(), "steam".to_owned());

        let fs = StubFilesystem::new()
            .with_file("/home/test/snap/steam/common/.local/share/Steam", "")
            .with_file(
                "/home/test/snap/steam/common/.local/share/Steam/steamapps/common/Proton 8.0",
                "",
            );

        let res = SteamCollector.collect(&ctx(StubCommandRunner(cmds), fs));

        let inst = res
            .facts
            .iter()
            .find(|f| f.id == ids::steam::INSTALLATION)
            .unwrap();
        assert!(inst.value.contains("Snap (Initialized in"));

        let proton = res
            .facts
            .iter()
            .find(|f| f.id == ids::steam::PROTON)
            .unwrap();
        assert_eq!(proton.value, "Snap Official: Proton 8.0");
    }

    #[test]
    fn both_native_and_flatpak_installed() {
        let mut cmds = HashMap::new();
        cmds.insert(
            "flatpak info com.valvesoftware.Steam".to_owned(),
            "installed".to_owned(),
        );

        let fs = StubFilesystem::new()
            .with_file("/usr/bin/steam", "")
            .with_file("/home/test/.local/share/Steam", "")
            .with_file("/home/test/.var/app/com.valvesoftware.Steam/data/Steam", "");

        let res = SteamCollector.collect(&ctx(StubCommandRunner(cmds), fs));

        let facts: Vec<_> = res
            .facts
            .iter()
            .filter(|f| f.id == ids::steam::INSTALLATION)
            .collect();
        assert_eq!(facts.len(), 2);
        assert!(facts[0].value.contains("Native"));
        assert!(facts[1].value.contains("Flatpak"));
    }
}
