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
}

#[cfg(test)]
mod tests {
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
    }
}
