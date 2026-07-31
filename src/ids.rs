<<<<<<< HEAD
//! Stable check identifier constants.
//!
//! Every check emitted by `GameDoc` carries a stable string identifier in the
//! `id` field of [`crate::model::Check`].  External consumers (scripts,
//! dashboards, integrations) may rely on these identifiers not changing across
//! patch releases.
//!
//! # Naming convention
//!
//! Identifiers follow the pattern `<category>.<name>` with `snake_case` for
//! both components.  Category names correspond to the variants of
//! [`crate::model::Category`].
//!
//! # Stability contract
//!
//! - IDs listed here are considered stable.
//! - New IDs may be added at any time.
//! - Existing IDs will not be renamed or removed without a major version bump.

/// Check IDs for the `system` category.
pub mod system {
    /// The running Linux kernel version string.
    pub const KERNEL_VERSION: &str = "system.kernel_version";
    /// CPU architecture (e.g. `x86_64`).
    pub const ARCHITECTURE: &str = "system.architecture";
    /// Display session type (`wayland` or `x11`).
    pub const SESSION_TYPE: &str = "system.session_type";
    /// Hostname detection (value intentionally withheld from output).
    pub const HOSTNAME: &str = "system.hostname";
    /// Total and available physical memory.
    pub const MEMORY: &str = "system.memory";
    /// Swap space configuration.
    pub const SWAP: &str = "system.swap";
    /// CPU I/O scheduler in use.
    pub const SCHEDULER: &str = "system.scheduler";
}

/// Check IDs for the `gpu` category.
pub mod gpu {
    pub const IDENTITY: &str = "gpu.identity";
    pub const VENDOR: &str = "gpu.vendor";
    pub const DRIVER: &str = "gpu.driver";
    pub const OPENGL: &str = "gpu.opengl";
    pub const VULKAN: &str = "gpu.vulkan";
    pub const RENDERER: &str = "gpu.renderer";
    pub const MESA: &str = "gpu.mesa";
    pub const NVIDIA: &str = "gpu.nvidia";
}

/// Check IDs for the `steam` category.
pub mod steam {
    /// Steam client installation.
    pub const INSTALLATION: &str = "steam.installation";
    /// Proton compatibility layer presence.
    pub const PROTON: &str = "steam.proton";
    /// Third-party Proton / compatibility tool directories.
    pub const COMPATIBILITY_TOOLS: &str = "steam.compatibility_tools";
}

/// Check IDs for the `mango_hud` category.
pub mod mangohud {
    /// `MangoHUD` installation detection.
    pub const INSTALLATION: &str = "mangohud.installation";
    /// `MangoHUD` configuration file presence.
    pub const CONFIG: &str = "mangohud.config";
}

/// Check IDs for the `game_mode` category.
pub mod gamemode {
    /// `GameMode` installation detection.
    pub const INSTALLATION: &str = "gamemode.installation";
    /// `GameMode` daemon running status.
    pub const DAEMON_RUNNING: &str = "gamemode.daemon_running";
=======
//! Stable, type-safe identifiers used across architecture boundaries.

use std::fmt;

use serde::Serialize;

macro_rules! identifier {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(&'static str);

        impl $name {
            #[must_use]
            pub const fn new(value: &'static str) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn as_str(self) -> &'static str {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.0)
            }
        }
    };
}

identifier!(CollectorId);
identifier!(FactId);
identifier!(ProblemId);
identifier!(RuleId);

pub mod collector {
    use super::CollectorId;

    pub const SYSTEM: CollectorId = CollectorId::new("system");
    pub const CPU: CollectorId = CollectorId::new("cpu");
    pub const DISTRO: CollectorId = CollectorId::new("distro");
    pub const GRAPHICS: CollectorId = CollectorId::new("graphics");
    pub const STEAM: CollectorId = CollectorId::new("steam");
    pub const MANGOHUD: CollectorId = CollectorId::new("mangohud");
    pub const GAMEMODE: CollectorId = CollectorId::new("gamemode");
}

pub mod fact {
    use super::FactId;

    pub const KERNEL_VERSION: FactId = FactId::new("system.kernel_version");
    pub const ARCHITECTURE: FactId = FactId::new("system.architecture");
    pub const SESSION_TYPE: FactId = FactId::new("system.session_type");
    pub const DESKTOP: FactId = FactId::new("system.desktop");
    pub const CPU_GOVERNOR: FactId = FactId::new("system.cpu_governor");
    pub const DISTRO_NAME: FactId = FactId::new("distro.name");
    pub const DISTRO_ID: FactId = FactId::new("distro.id");
    pub const DISTRO_VERSION_ID: FactId = FactId::new("distro.version_id");
    pub const DISTRO_PRETTY_NAME: FactId = FactId::new("distro.pretty_name");
    pub const DISTRO_FAMILY: FactId = FactId::new("distro.family");
    pub const CPU_VENDOR: FactId = FactId::new("cpu.vendor");
    pub const CPU_MODEL: FactId = FactId::new("cpu.model");
    pub const CPU_CORES_LOGICAL: FactId = FactId::new("cpu.cores_logical");
    pub const CPU_CORES_PHYSICAL: FactId = FactId::new("cpu.cores_physical");
    pub const CPU_FREQUENCY_MHZ: FactId = FactId::new("cpu.frequency_mhz");
    pub const CPU_VIRTUALIZATION_FLAGS: FactId = FactId::new("cpu.virtualization_flags");
    pub const CPU_ARCHITECTURE: FactId = FactId::new("cpu.architecture");
    pub const GPU_DEVICE: FactId = FactId::new("gpu.device");
    pub const VULKAN_AVAILABLE: FactId = FactId::new("gpu.vulkan_available");
    pub const OPENGL_RENDERER: FactId = FactId::new("gpu.opengl_renderer");
    pub const OPENGL_VERSION: FactId = FactId::new("gpu.opengl_version");
    pub const STEAM_INSTALLATION: FactId = FactId::new("steam.installation");
    pub const STEAM_EXECUTABLE: FactId = FactId::new("steam.executable");
    pub const STEAM_DATA_READABLE: FactId = FactId::new("steam.data_readable");
    pub const STEAM_LIBRARY: FactId = FactId::new("steam.library");
    pub const STEAM_LIBRARY_READABLE: FactId = FactId::new("steam.library_readable");
    pub const STEAM_PROTON_VERSION: FactId = FactId::new("steam.proton_version");
    pub const STEAM_PROTON_SCAN_COMPLETE: FactId = FactId::new("steam.proton_scan_complete");
    pub const STEAM_COMPATIBILITY_TOOL: FactId = FactId::new("steam.compatibility_tool");
    pub const MANGOHUD_EXECUTABLE: FactId = FactId::new("mangohud.executable");
    pub const MANGOHUD_INSTALLATION: FactId = FactId::new("mangohud.installation");
    pub const MANGOHUD_CONFIG: FactId = FactId::new("mangohud.config");
    pub const MANGOHUD_CONFIG_KEY: FactId = FactId::new("mangohud.config_key");
    pub const MANGOHUD_CONFIG_GLOBAL: FactId = FactId::new("mangohud.config_global");
    pub const MANGOHUD_ENVIRONMENT: FactId = FactId::new("mangohud.environment_evidence");
    pub const GAMEMODE_EXECUTABLE: FactId = FactId::new("gamemode.executable");
    pub const GAMEMODE_DAEMON_AVAILABLE: FactId = FactId::new("gamemode.daemon_available");
    pub const GAMEMODE_DAEMON_RUNNING: FactId = FactId::new("gamemode.daemon_running");
    pub const GAMEMODE_LIBRARY: FactId = FactId::new("gamemode.library");
    pub const GAMEMODE_CONFIG: FactId = FactId::new("gamemode.config");
}

pub mod rule {
    use super::RuleId;

    pub const VULKAN_AVAILABLE: RuleId = RuleId::new("gpu.vulkan_available");
    pub const STEAM_PROTON: RuleId = RuleId::new("steam.proton_runtime");
    pub const STEAM_LIBRARY_READABLE: RuleId = RuleId::new("steam.library_readable");
    pub const GAMEMODE_DAEMON: RuleId = RuleId::new("gamemode.daemon_available");
}

pub mod problem {
    use super::ProblemId;

    pub const VULKAN_UNAVAILABLE: ProblemId = ProblemId::new("gpu.vulkan_unavailable");
    pub const STEAM_PROTON_MISSING: ProblemId = ProblemId::new("steam.proton_missing");
    pub const STEAM_LIBRARY_UNREADABLE: ProblemId = ProblemId::new("steam.library_unreadable");
    pub const GAMEMODE_DAEMON_UNAVAILABLE: ProblemId =
        ProblemId::new("gamemode.daemon_unavailable");
>>>>>>> 2109919 (v0.2.0)
}

#[cfg(test)]
mod tests {
<<<<<<< HEAD
    use super::{gamemode, gpu, mangohud, steam, system};

    /// All IDs must be non-empty and follow the `category.name` pattern.
    fn assert_id_well_formed(id: &str) {
        assert!(!id.is_empty(), "ID must not be empty");
        assert!(id.contains('.'), "ID `{id}` must contain a dot separator");
        assert!(id == id.to_ascii_lowercase(), "ID `{id}` must be lowercase");
        assert!(
            !id.starts_with('.') && !id.ends_with('.'),
            "ID `{id}` must not start or end with a dot"
        );
    }

    #[test]
    fn all_system_ids_are_well_formed() {
        for id in [
            system::KERNEL_VERSION,
            system::ARCHITECTURE,
            system::SESSION_TYPE,
            system::HOSTNAME,
            system::MEMORY,
            system::SWAP,
            system::SCHEDULER,
        ] {
            assert_id_well_formed(id);
        }
    }

    #[test]
    fn all_gpu_ids_are_well_formed() {
        for id in [
            gpu::IDENTITY,
            gpu::VENDOR,
            gpu::DRIVER,
            gpu::OPENGL,
            gpu::VULKAN,
            gpu::RENDERER,
            gpu::MESA,
            gpu::NVIDIA,
        ] {
            assert_id_well_formed(id);
        }
    }

    #[test]
    fn all_steam_ids_are_well_formed() {
        for id in [
            steam::INSTALLATION,
            steam::PROTON,
            steam::COMPATIBILITY_TOOLS,
        ] {
            assert_id_well_formed(id);
        }
    }

    #[test]
    fn all_mangohud_ids_are_well_formed() {
        for id in [mangohud::INSTALLATION, mangohud::CONFIG] {
            assert_id_well_formed(id);
        }
    }

    #[test]
    fn all_gamemode_ids_are_well_formed() {
        for id in [gamemode::INSTALLATION, gamemode::DAEMON_RUNNING] {
            assert_id_well_formed(id);
        }
    }

    #[test]
    fn system_ids_have_correct_prefix() {
        for id in [system::KERNEL_VERSION, system::ARCHITECTURE, system::MEMORY] {
            assert!(id.starts_with("system."), "expected system. prefix: {id}");
        }
=======
    use super::{CollectorId, FactId};

    #[test]
    fn distinct_identifier_types_have_stable_strings() {
        let collector = CollectorId::new("system");
        let fact = FactId::new("system.kernel_version");

        assert_eq!(collector.as_str(), "system");
        assert_eq!(fact.as_str(), "system.kernel_version");
>>>>>>> 2109919 (v0.2.0)
    }
}
