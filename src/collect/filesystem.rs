//! Filesystem access abstraction.
//!
//! All file reads in `GameDoc` go through [`Filesystem`] rather than calling
//! [`std::fs`] directly.  This keeps every collector independently testable:
//! tests inject a fake [`Filesystem`] populated with fixture data instead of
//! reading from the real `/proc`, `/sys`, or `$HOME`.
//!
//! # Design rules
//!
//! - `GameDoc` never writes to the filesystem.
//! - [`Filesystem::read_to_string`] is the primary read primitive.
//! - [`Filesystem::read_dir_names`] returns sorted, deterministic results.
//! - Unknown or missing files are handled at the caller level; implementations
//!   return errors rather than silently returning empty strings.

use std::path::{Path, PathBuf};

use crate::errors::CollectError;

/// Read-only access to the local filesystem.
pub trait Filesystem: Send + Sync {
    /// Read the entire contents of a file as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns [`CollectError::FilesystemRead`] if the file cannot be opened
    /// or read.
    fn read_to_string(&self, path: &Path) -> Result<String, CollectError>;

    /// Return `true` if `path` refers to an existing file or directory.
    fn exists(&self, path: &Path) -> bool;

    /// List the names (not full paths) of entries in a directory.
    ///
    /// Results are sorted for determinism.  Entries whose names are not valid
    /// UTF-8 are silently skipped.
    ///
    /// # Errors
    ///
    /// Returns [`CollectError::FilesystemRead`] if the directory cannot be
    /// opened.
    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, CollectError>;
}

/// A [`Filesystem`] implementation that reads from the real local filesystem.
#[derive(Clone, Debug, Default)]
pub struct LocalFilesystem;

impl Filesystem for LocalFilesystem {
    fn read_to_string(&self, path: &Path) -> Result<String, CollectError> {
        std::fs::read_to_string(path).map_err(|source| CollectError::FilesystemRead {
            path: path.to_owned(),
            source,
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, CollectError> {
        let entries = std::fs::read_dir(path).map_err(|source| CollectError::FilesystemRead {
            path: path.to_owned(),
            source,
        })?;

        let mut names: Vec<String> = entries
            .filter_map(std::result::Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();

        // Sort for determinism: callers must not rely on filesystem ordering.
        names.sort();
        Ok(names)
    }
}

/// A [`Filesystem`] backed by an in-memory map of path → content.
///
/// Intended for use in unit tests.  Paths that are not in the map behave as
/// if they do not exist.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use gamedoc::collect::filesystem::{Filesystem, StubFilesystem};
///
/// let fs = StubFilesystem::new()
///     .with_file("/proc/version", "Linux version 6.9.0");
///
/// assert!(fs.exists(Path::new("/proc/version")));
/// assert!(!fs.exists(Path::new("/proc/nonexistent")));
/// ```
#[derive(Clone, Debug, Default)]
pub struct StubFilesystem {
    files: std::collections::BTreeMap<PathBuf, String>,
}

impl StubFilesystem {
    /// Create an empty stub filesystem.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file at `path` with the given `content` and return `self`.
    #[must_use]
    pub fn with_file(mut self, path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        self.files.insert(path.into(), content.into());
        self
    }
}

impl Filesystem for StubFilesystem {
    fn read_to_string(&self, path: &Path) -> Result<String, CollectError> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| CollectError::FilesystemRead {
                path: path.to_owned(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, CollectError> {
        // Return names of all files whose parent matches `path`.
        let mut names: Vec<String> = self
            .files
            .keys()
            .filter(|p| p.parent() == Some(path))
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_owned))
            .collect();

        names.sort();

        if names.is_empty() && !self.files.keys().any(|p| p.starts_with(path)) {
            return Err(CollectError::FilesystemRead {
                path: path.to_owned(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            });
        }

        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Filesystem, LocalFilesystem, StubFilesystem};
    use crate::errors::CollectError;

    // --- StubFilesystem tests ---

    #[test]
    fn stub_read_returns_content() {
        let fs = StubFilesystem::new().with_file("/proc/version", "Linux version 6.9.0");
        let content = fs
            .read_to_string(Path::new("/proc/version"))
            .expect("stub should return content");
        assert_eq!(content, "Linux version 6.9.0");
    }

    #[test]
    fn stub_read_missing_returns_error() {
        let fs = StubFilesystem::new();
        let result = fs.read_to_string(Path::new("/proc/nonexistent"));
        assert!(
            matches!(result, Err(CollectError::FilesystemRead { .. })),
            "expected FilesystemRead error"
        );
    }

    #[test]
    fn stub_exists_present_file() {
        let fs = StubFilesystem::new().with_file("/dev/dri/renderD128", "");
        assert!(fs.exists(Path::new("/dev/dri/renderD128")));
    }

    #[test]
    fn stub_exists_absent_file() {
        let fs = StubFilesystem::new();
        assert!(!fs.exists(Path::new("/dev/dri/renderD128")));
    }

    #[test]
    fn stub_read_dir_names_returns_sorted_names() {
        let fs = StubFilesystem::new()
            .with_file("/dev/dri/renderD129", "")
            .with_file("/dev/dri/renderD128", "")
            .with_file("/dev/dri/card0", "");

        let names = fs
            .read_dir_names(Path::new("/dev/dri"))
            .expect("stub dir should be readable");

        assert_eq!(names, vec!["card0", "renderD128", "renderD129"]);
    }

    #[test]
    fn stub_read_dir_missing_returns_error() {
        let fs = StubFilesystem::new();
        let result = fs.read_dir_names(Path::new("/nonexistent/dir"));
        assert!(
            matches!(result, Err(CollectError::FilesystemRead { .. })),
            "expected FilesystemRead error"
        );
    }

    // --- LocalFilesystem smoke tests (require /proc) ---

    #[test]
    fn local_fs_reads_proc_version() {
        let fs = LocalFilesystem;
        let content = fs
            .read_to_string(Path::new("/proc/version"))
            .expect("/proc/version should be readable on Linux");
        assert!(
            content.contains("Linux"),
            "/proc/version should contain 'Linux'"
        );
    }

    #[test]
    fn local_fs_exists_proc() {
        let fs = LocalFilesystem;
        assert!(fs.exists(Path::new("/proc")));
        assert!(!fs.exists(Path::new("/proc/totally-nonexistent-xyz")));
    }

    #[test]
    fn local_fs_read_nonexistent_returns_error() {
        let fs = LocalFilesystem;
        let result = fs.read_to_string(Path::new("/proc/totally-nonexistent-file-xyz"));
        assert!(
            matches!(result, Err(CollectError::FilesystemRead { .. })),
            "expected FilesystemRead error"
        );
    }
}
