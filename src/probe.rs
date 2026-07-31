//! The complete read-only platform I/O boundary.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::collect::command::{CommandOutput, CommandRunner, ProcessCommandRunner};
use crate::errors::CollectError;

pub trait SystemProbe: Send + Sync {
    /// Execute one program with an explicit argument vector.
    ///
    /// # Errors
    ///
    /// Returns a collection error when the program cannot be executed or
    /// exceeds the probe timeout.
    fn command(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CollectError>;

    /// Read one file as text.
    ///
    /// # Errors
    ///
    /// Returns a collection error when the path cannot be read.
    fn read(&self, path: &Path) -> Result<String, CollectError>;

    /// Read sorted names from one directory.
    ///
    /// # Errors
    ///
    /// Returns a collection error when the directory cannot be read.
    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, CollectError>;
    fn exists(&self, path: &Path) -> bool;
    fn which(&self, program: &str) -> Option<PathBuf>;
    fn env(&self, name: &str) -> Option<OsString>;
}

#[derive(Clone, Debug, Default)]
pub struct LocalSystemProbe {
    command: ProcessCommandRunner,
}

impl LocalSystemProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl SystemProbe for LocalSystemProbe {
    fn command(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CollectError> {
        self.command.run(program, args)
    }

    fn read(&self, path: &Path) -> Result<String, CollectError> {
        std::fs::read_to_string(path).map_err(|source| CollectError::FilesystemRead {
            path: path.to_owned(),
            source,
        })
    }

    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, CollectError> {
        let entries = std::fs::read_dir(path).map_err(|source| CollectError::FilesystemRead {
            path: path.to_owned(),
            source,
        })?;
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();
        Ok(names)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn which(&self, program: &str) -> Option<PathBuf> {
        let program = Path::new(program);
        if program.components().count() > 1 {
            return program.is_file().then(|| program.to_owned());
        }

        std::env::split_paths(&std::env::var_os("PATH")?)
            .map(|directory| directory.join(program))
            .find(|candidate| candidate.is_file() && is_executable(candidate))
    }

    fn env(&self, name: &str) -> Option<OsString> {
        std::env::var_os(name)
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    path.metadata()
        .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[derive(Clone, Debug, Default)]
pub struct FixtureSystemProbe {
    commands: BTreeMap<(String, Vec<String>), Result<CommandOutput, FixtureError>>,
    files: BTreeMap<PathBuf, String>,
    directories: BTreeMap<PathBuf, Vec<String>>,
    unreadable: BTreeSet<PathBuf>,
    executables: BTreeMap<String, PathBuf>,
    environment: BTreeMap<String, OsString>,
}

#[derive(Clone, Debug)]
enum FixtureError {
    CommandNotFound,
}

impl FixtureSystemProbe {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_command(mut self, program: &str, args: &[&str], output: CommandOutput) -> Self {
        self.commands.insert(command_key(program, args), Ok(output));
        self
    }

    #[must_use]
    pub fn with_missing_command(mut self, program: &str, args: &[&str]) -> Self {
        self.commands.insert(
            command_key(program, args),
            Err(FixtureError::CommandNotFound),
        );
        self
    }

    #[must_use]
    pub fn with_file(mut self, path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        self.files.insert(path.into(), content.into());
        self
    }

    #[must_use]
    pub fn with_directory(mut self, path: impl Into<PathBuf>, names: Vec<String>) -> Self {
        let mut names = names;
        names.sort();
        self.directories.insert(path.into(), names);
        self
    }

    #[must_use]
    pub fn with_unreadable_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.unreadable.insert(path.into());
        self
    }

    #[must_use]
    pub fn with_executable(mut self, program: &str, path: impl Into<PathBuf>) -> Self {
        self.executables.insert(program.to_owned(), path.into());
        self
    }

    #[must_use]
    pub fn with_env(mut self, name: &str, value: impl Into<OsString>) -> Self {
        self.environment.insert(name.to_owned(), value.into());
        self
    }
}

impl SystemProbe for FixtureSystemProbe {
    fn command(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CollectError> {
        match self.commands.get(&command_key(program, args)) {
            Some(Ok(output)) => Ok(output.clone()),
            Some(Err(FixtureError::CommandNotFound)) | None => Err(CollectError::CommandNotFound {
                program: program.to_owned(),
            }),
        }
    }

    fn read(&self, path: &Path) -> Result<String, CollectError> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| fixture_read_error(path))
    }

    fn read_dir_names(&self, path: &Path) -> Result<Vec<String>, CollectError> {
        if self.unreadable.contains(path) {
            return Err(CollectError::FilesystemRead {
                path: path.to_owned(),
                source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
            });
        }
        self.directories
            .get(path)
            .cloned()
            .ok_or_else(|| fixture_read_error(path))
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
            || self.directories.contains_key(path)
            || self.unreadable.contains(path)
    }

    fn which(&self, program: &str) -> Option<PathBuf> {
        self.executables.get(program).cloned()
    }

    fn env(&self, name: &str) -> Option<OsString> {
        self.environment.get(name).cloned()
    }
}

fn command_key(program: &str, args: &[&str]) -> (String, Vec<String>) {
    (
        program.to_owned(),
        args.iter().map(|argument| (*argument).to_owned()).collect(),
    )
}

fn fixture_read_error(path: &Path) -> CollectError {
    CollectError::FilesystemRead {
        path: path.to_owned(),
        source: std::io::Error::from(std::io::ErrorKind::NotFound),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{FixtureSystemProbe, SystemProbe};
    use crate::collect::command::CommandOutput;

    #[test]
    fn fixture_returns_exact_command_match() {
        let fixture = FixtureSystemProbe::new().with_command(
            "uname",
            &["-r"],
            CommandOutput {
                stdout: "6.9.0\n".to_owned(),
                stderr: String::new(),
                exit_code: Some(0),
            },
        );

        assert_eq!(fixture.command("uname", &["-r"]).unwrap().stdout, "6.9.0\n");
        assert!(fixture.command("uname", &["-a"]).is_err());
    }

    #[test]
    fn fixture_directory_order_is_deterministic() {
        let fixture = FixtureSystemProbe::new().with_directory(
            "/sys/class/drm",
            vec!["card1".to_owned(), "card0".to_owned()],
        );

        assert_eq!(
            fixture.read_dir_names(Path::new("/sys/class/drm")).unwrap(),
            ["card0", "card1"]
        );
    }
}
