//! Collector execution context.
//!
//! [`CollectorContext`] is the single object passed into every
//! [`super::Collector::collect`] call.  It bundles all external dependencies
//! that collectors need: a command runner, a filesystem, and (in future
//! phases) environment variables, a clock, and other injectable services.
//!
//! # Testability
//!
//! Because every dependency is accessed through a trait, unit tests can
//! construct a `CollectorContext` with fake implementations:
//!
//! ```rust
//! use std::sync::Arc;
//! use gamedoc::collect::context::CollectorContext;
//! use gamedoc::collect::command::ProcessCommandRunner;
//! use gamedoc::collect::filesystem::StubFilesystem;
//!
//! let ctx = CollectorContext::new(
//!     Arc::new(ProcessCommandRunner::new()),
//!     Arc::new(StubFilesystem::new().with_file("/proc/version", "Linux version 6.9.0")),
//! );
//! ```

use std::sync::Arc;

use super::command::{CommandRunner, ProcessCommandRunner};
use super::filesystem::{Filesystem, LocalFilesystem};

/// Bundles all external dependencies for a [`super::Collector`].
///
/// Collectors receive a reference to this context and must not access any
/// external resource (filesystem, commands, environment) except through it.
pub struct CollectorContext {
    /// The command runner used to invoke external tools.
    pub command: Arc<dyn CommandRunner>,
    /// The filesystem used to read local files and directories.
    pub fs: Arc<dyn Filesystem>,
}

impl CollectorContext {
    /// Create a context with the provided implementations.
    #[must_use]
    pub fn new(command: Arc<dyn CommandRunner>, fs: Arc<dyn Filesystem>) -> Self {
        Self { command, fs }
    }

    /// Create a context wired to the real local system.
    ///
    /// This is the context used in production.  Tests should prefer
    /// [`CollectorContext::new`] with injected fakes.
    #[must_use]
    pub fn real() -> Self {
        Self {
            command: Arc::new(ProcessCommandRunner::new()),
            fs: Arc::new(LocalFilesystem),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use super::CollectorContext;
    use crate::collect::filesystem::StubFilesystem;
    use crate::errors::CollectError;

    #[test]
    fn real_context_can_run_echo() {
        let ctx = CollectorContext::real();
        let output = ctx
            .command
            .run("echo", &["gamedoc"])
            .expect("echo should succeed");
        assert_eq!(output.stdout.trim(), "gamedoc");
    }

    #[test]
    fn injected_stub_filesystem_is_used() {
        let stub = StubFilesystem::new().with_file("/proc/version", "Linux version 6.9.0 stub");

        let ctx = CollectorContext::new(
            Arc::new(crate::collect::command::ProcessCommandRunner::new()),
            Arc::new(stub),
        );

        let content = ctx
            .fs
            .read_to_string(Path::new("/proc/version"))
            .expect("stub should return content");

        assert!(content.contains("stub"));
    }

    #[test]
    fn injected_stub_returns_not_found_for_missing_files() {
        let ctx = CollectorContext::new(
            Arc::new(crate::collect::command::ProcessCommandRunner::new()),
            Arc::new(StubFilesystem::new()),
        );

        let result = ctx.fs.read_to_string(Path::new("/proc/version"));
        assert!(
            matches!(result, Err(CollectError::FilesystemRead { .. })),
            "expected FilesystemRead, got {result:?}"
        );
    }
}
