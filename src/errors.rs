//! Structured error types for the collection layer.
//!
//! All errors that can occur during diagnostics collection are represented
//! here as a single [`CollectError`] enum.  This keeps the error surface of
//! the library predictable and documented.
//!
//! Rendering code uses [`anyhow`] for propagation; only collection code should
//! produce `CollectError` values directly.

use std::path::PathBuf;
use std::time::Duration;

/// All errors that can originate in the collection layer.
#[derive(Debug, thiserror::Error)]
pub enum CollectError {
    /// The requested executable was not found on `PATH`.
    #[error("command not found: {program}")]
    CommandNotFound {
        /// The program that could not be located.
        program: String,
    },

    /// A command ran longer than the configured timeout and was killed.
    #[error("command timed out after {timeout:?}: {program} {args}")]
    CommandTimeout {
        /// The program that timed out.
        program: String,
        /// The arguments passed to the program, joined with spaces.
        args: String,
        /// The timeout that was exceeded.
        timeout: Duration,
    },

    /// The operating system returned an error while spawning or waiting for a
    /// child process.
    #[error("I/O error running command: {source}")]
    CommandIo {
        /// The underlying OS error.
        #[source]
        source: std::io::Error,
    },

    /// A file or directory could not be read.
    #[error("failed to read {path}: {source}")]
    FilesystemRead {
        /// The path that could not be read.
        path: PathBuf,
        /// The underlying OS error.
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::CollectError;

    #[test]
    fn command_not_found_formats_correctly() {
        let err = CollectError::CommandNotFound {
            program: "nonexistent".to_owned(),
        };
        assert!(err.to_string().contains("nonexistent"));
    }

    #[test]
    fn command_timeout_formats_correctly() {
        let err = CollectError::CommandTimeout {
            program: "slowcmd".to_owned(),
            args: "--wait".to_owned(),
            timeout: Duration::from_secs(5),
        };
        let s = err.to_string();
        assert!(s.contains("slowcmd"));
        assert!(s.contains("5s"));
    }

    #[test]
    fn filesystem_read_error_includes_path() {
        let err = CollectError::FilesystemRead {
            path: PathBuf::from("/proc/nonexistent"),
            source: std::io::Error::from(std::io::ErrorKind::NotFound),
        };
        assert!(err.to_string().contains("/proc/nonexistent"));
    }
}
