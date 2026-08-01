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
    pub const GAMESCOPE_INSTALLATION: FactId = FactId::new("gamescope.installation");
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
    pub const CPU_GOVERNOR: RuleId = RuleId::new("system.cpu_governor");
}

pub mod problem {
    use super::ProblemId;

    pub const VULKAN_UNAVAILABLE: ProblemId = ProblemId::new("gpu.vulkan_unavailable");
    pub const STEAM_PROTON_MISSING: ProblemId = ProblemId::new("steam.proton_missing");
    pub const STEAM_LIBRARY_UNREADABLE: ProblemId = ProblemId::new("steam.library_unreadable");
    pub const GAMEMODE_DAEMON_UNAVAILABLE: ProblemId =
        ProblemId::new("gamemode.daemon_unavailable");
    pub const CPU_GOVERNOR_POWERSAVE: ProblemId = ProblemId::new("system.cpu_governor_powersave");
}

#[cfg(test)]
mod tests {
    use super::{CollectorId, FactId};

    #[test]
    fn distinct_identifier_types_have_stable_strings() {
        let collector = CollectorId::new("system");
        let fact = FactId::new("system.kernel_version");

        assert_eq!(collector.as_str(), "system");
        assert_eq!(fact.as_str(), "system.kernel_version");
    }
}
