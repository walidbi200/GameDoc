//! Command execution abstraction.
//!
//! All external command invocations in `GameDoc` go through [`CommandRunner`].
//! This keeps command execution centrally governed (timeout, argv-only, no
//! shell) and makes every collector independently testable by injecting a
//! [`CommandRunner`] implementation that returns canned output.
//!
//! # Design rules
//!
//! - Commands are always specified as an explicit program + argument slice.
//! - `sh -c` and shell pipelines are **never** used.
//! - Every invocation has an enforced wall-clock timeout.
//! - `stdout` and `stderr` are always captured separately.

use std::io::Read as _;
use std::thread;
use std::time::Duration;

use wait_timeout::ChildExt as _;

use crate::errors::CollectError;

/// The default timeout applied to every command invocation.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// The captured output of a completed command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    /// Contents of the command's standard output, decoded as UTF-8 lossy.
    pub stdout: String,
    /// Contents of the command's standard error, decoded as UTF-8 lossy.
    pub stderr: String,
    /// Exit code, or `None` if the process was terminated by a signal.
    pub exit_code: Option<i32>,
}

impl CommandOutput {
    /// Returns `true` if the command exited with status zero.
    #[must_use]
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Runs external commands and returns their captured output.
///
/// Implementations must:
///
/// - Never invoke a shell.
/// - Always capture both `stdout` and `stderr`.
/// - Enforce a timeout and return [`CollectError::CommandTimeout`] when it
///   elapses.
pub trait CommandRunner: Send + Sync {
    /// Run `program` with `args` and return the captured output.
    ///
    /// # Errors
    ///
    /// - [`CollectError::CommandNotFound`] — `program` is not on `PATH`.
    /// - [`CollectError::CommandTimeout`] — the command exceeded the timeout.
    /// - [`CollectError::CommandIo`] — the OS rejected the spawn request.
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CollectError>;
}

/// A [`CommandRunner`] that executes real child processes.
///
/// Uses `wait_timeout` to enforce wall-clock timeouts without busy-waiting.
/// When the timeout elapses the child is killed and reaped before
/// [`CollectError::CommandTimeout`] is returned.
///
/// Standard output and standard error are read on dedicated threads so large
/// output cannot block process exit.
#[derive(Clone, Debug)]
pub struct ProcessCommandRunner {
    timeout: Duration,
}

impl ProcessCommandRunner {
    /// Create a runner with the [`DEFAULT_TIMEOUT`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Create a runner with a custom timeout.
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for ProcessCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner for ProcessCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, CollectError> {
        use std::process::{Command, Stdio};

        let mut child = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CollectError::CommandNotFound {
                        program: program.to_owned(),
                    }
                } else {
                    CollectError::CommandIo { source: e }
                }
            })?;

        let stdout = child.stdout.take().map(|mut pipe| {
            thread::spawn(move || {
                let mut bytes = Vec::new();
                pipe.read_to_end(&mut bytes).map(|_| bytes)
            })
        });
        let stderr = child.stderr.take().map(|mut pipe| {
            thread::spawn(move || {
                let mut bytes = Vec::new();
                pipe.read_to_end(&mut bytes).map(|_| bytes)
            })
        });

        let Some(exit_status) = child
            .wait_timeout(self.timeout)
            .map_err(|source| CollectError::CommandIo { source })?
        else {
            child.kill().ok();
            child.wait().ok();
            return Err(CollectError::CommandTimeout {
                program: program.to_owned(),
                args: args.join(" "),
                timeout: self.timeout,
            });
        };

        let stdout = join_output(stdout)?;
        let stderr = join_output(stderr)?;

        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code: exit_status.code(),
        })
    }
}

fn join_output(
    reader: Option<thread::JoinHandle<std::io::Result<Vec<u8>>>>,
) -> Result<String, CollectError> {
    let Some(reader) = reader else {
        return Ok(String::new());
    };
    let bytes = reader
        .join()
        .map_err(|_| CollectError::CommandCaptureThread)?
        .map_err(|source| CollectError::CommandIo { source })?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{CommandOutput, CommandRunner, ProcessCommandRunner};
    use crate::errors::CollectError;

    fn runner() -> ProcessCommandRunner {
        ProcessCommandRunner::new()
    }

    #[test]
    fn echo_captures_stdout() {
        let output = runner()
            .run("echo", &["hello"])
            .expect("echo should succeed");
        assert!(
            output.stdout.trim() == "hello",
            "expected 'hello', got {:?}",
            output.stdout
        );
        assert!(output.success());
    }

    #[test]
    fn missing_command_returns_not_found() {
        let result = runner().run("gamedoc-totally-nonexistent-command-xyz", &[]);
        assert!(
            matches!(result, Err(CollectError::CommandNotFound { .. })),
            "expected CommandNotFound, got {result:?}"
        );
    }

    #[test]
    fn stderr_is_captured_separately() {
        // `ls` on a non-existent path writes to stderr and exits non-zero.
        let output = runner()
            .run("ls", &["/totally/nonexistent/path/xyz"])
            .expect("ls spawn should succeed");
        assert!(!output.success());
        assert!(
            !output.stderr.is_empty(),
            "expected stderr to contain error message"
        );
    }

    #[test]
    fn timeout_returns_timeout_error() {
        let short = ProcessCommandRunner::with_timeout(Duration::from_millis(100));
        // `sleep 10` will exceed the 100 ms timeout.
        let result = short.run("sleep", &["10"]);
        assert!(
            matches!(result, Err(CollectError::CommandTimeout { .. })),
            "expected CommandTimeout, got {result:?}"
        );
    }

    #[test]
    fn success_returns_correct_exit_code() {
        let output = runner().run("true", &[]).expect("true should succeed");
        assert_eq!(output.exit_code, Some(0));
    }

    #[test]
    fn failure_returns_nonzero_exit_code() {
        let output = runner().run("false", &[]).expect("false should spawn");
        assert_ne!(output.exit_code, Some(0));
    }

    #[test]
    fn command_output_success_reflects_exit_code() {
        let passing = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
        };
        let failing = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(1),
        };
        let signalled = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
        };

        assert!(passing.success());
        assert!(!failing.success());
        assert!(!signalled.success());
    }

    #[test]
    fn large_output_does_not_block_child_exit() {
        let output = runner()
            .run("seq", &["1", "50000"])
            .expect("large output should be drained while the child runs");

        assert!(output.success());
        assert!(output.stdout.len() > 64 * 1024);
    }

    #[test]
    fn invalid_utf8_is_decoded_lossily() {
        let output = runner()
            .run("printf", &["\\377"])
            .expect("invalid UTF-8 output should still be captured");

        assert!(output.stdout.contains('\u{fffd}'));
    }
}
