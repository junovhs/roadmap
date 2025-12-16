//! Verification Runner: Executes shell commands to verify task completion.
//!
//! The core principle: A task is not PROVEN until `verify_cmd` returns Exit Code 0.

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
            timeout_secs: 300,
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

        Ok(VerifyResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }

    /// Runs verification with user-friendly output on failure.
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

/// Gets current git HEAD SHA.
pub fn get_git_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let runner = VerifyRunner::default_runner();
        let result = runner.run("echo hello").expect("run failed");
        assert!(result.passed());
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_failing_command() {
        let runner = VerifyRunner::default_runner();
        let result = runner.run("exit 1").expect("run failed");
        assert!(!result.passed());
        assert_eq!(result.exit_code, Some(1));
    }
}
