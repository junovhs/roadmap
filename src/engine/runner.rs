//! Verification Runner: Executes shell commands to verify task completion.
//!
//! The core principle: A task is not DONE until `verify_cmd` returns Exit Code 0.

use anyhow::{bail, Context, Result};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// Result of running a verification command.
#[derive(Debug)]
pub struct VerifyResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl VerifyResult {
    /// Returns true if the verification passed (exit code 0).
    #[must_use]
    pub fn passed(&self) -> bool {
        self.success && self.exit_code == Some(0)
    }
}

/// Configuration for the verification runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub timeout_secs: u64,
    pub capture_output: bool,
    pub working_dir: Option<String>,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 300, // 5 minutes default
            capture_output: true,
            working_dir: None,
        }
    }
}

/// Executes verification commands.
pub struct VerifyRunner {
    config: RunnerConfig,
}

impl VerifyRunner {
    #[must_use]
    pub fn new(config: RunnerConfig) -> Self {
        Self { config }
    }

    /// Creates a runner with default configuration.
    #[must_use]
    pub fn default_runner() -> Self {
        Self::new(RunnerConfig::default())
    }

    /// Executes a shell command and returns the result.
    ///
    /// # Errors
    /// Returns error if command fails to spawn.
    pub fn run(&self, cmd: &str) -> Result<VerifyResult> {
        if cmd.trim().is_empty() {
            bail!("Empty verification command");
        }

        let start = Instant::now();

        let shell = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        let mut command = Command::new(shell.0);
        command.arg(shell.1).arg(cmd);

        if self.config.capture_output {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        if let Some(ref dir) = self.config.working_dir {
            command.current_dir(dir);
        }

        let output: Output = command
            .spawn()
            .context("Failed to spawn verification command")?
            .wait_with_output()
            .context("Failed to wait for command")?;

        let duration = start.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(VerifyResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout,
            stderr,
            duration,
        })
    }

    /// Runs verification and returns a user-friendly status.
    ///
    /// # Errors
    /// Returns error if command fails to execute.
    pub fn verify(&self, cmd: &str) -> Result<VerifyResult> {
        let result = self.run(cmd)?;

        if !result.passed() {
            eprintln!("╭─ Verification Failed ─────────────────────────");
            eprintln!("│ Command: {cmd}");
            if let Some(code) = result.exit_code {
                eprintln!("│ Exit Code: {code}");
            }
            if !result.stderr.is_empty() {
                eprintln!("│ Stderr:");
                for line in result.stderr.lines().take(10) {
                    eprintln!("│   {line}");
                }
            }
            eprintln!("╰────────────────────────────────────────────────");
        }

        Ok(result)
    }
}

/// Common verification commands for different project types.
pub struct VerifyTemplates;

impl VerifyTemplates {
    /// Returns a test command template for Rust projects.
    #[must_use]
    pub fn rust_test(test_name: &str) -> String {
        format!("cargo test {test_name} --quiet")
    }

    /// Returns a test command template for Node.js projects.
    #[must_use]
    pub fn node_test(test_pattern: &str) -> String {
        format!("npm test -- --grep \"{test_pattern}\"")
    }

    /// Returns a test command template for Python projects.
    #[must_use]
    pub fn python_test(test_name: &str) -> String {
        format!("python -m pytest -q -k \"{test_name}\"")
    }

    /// Returns a simple file existence check.
    #[must_use]
    pub fn file_exists(path: &str) -> String {
        if cfg!(target_os = "windows") {
            format!("if exist \"{path}\" (exit 0) else (exit 1)")
        } else {
            format!("test -f \"{path}\"")
        }
    }

    /// Returns a build success check for Rust.
    #[must_use]
    pub fn rust_build() -> String {
        "cargo build --quiet".to_string()
    }

    /// Returns a lint check for Rust.
    #[must_use]
    pub fn rust_clippy() -> String {
        "cargo clippy --quiet -- -D warnings".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let runner = VerifyRunner::default_runner();
        let result = runner.run("echo hello").unwrap_or_else(|_| panic!("run failed"));
        assert!(result.passed());
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_failing_command() {
        let runner = VerifyRunner::default_runner();
        let result = runner.run("exit 1").unwrap_or_else(|_| panic!("run failed"));
        assert!(!result.passed());
        assert_eq!(result.exit_code, Some(1));
    }

    #[test]
    fn test_empty_command() {
        let runner = VerifyRunner::default_runner();
        let result = runner.run("");
        assert!(result.is_err());
    }
}
