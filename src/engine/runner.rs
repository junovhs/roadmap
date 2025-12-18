//! Verification Runner: Executes shell commands to verify task completion.

use anyhow::{bail, Context, Result};
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

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
    /// Returns error if command fails to spawn or times out.
    pub fn run(&self, cmd: &str) -> Result<VerifyResult> {
        if cmd.trim().is_empty() {
            bail!("Empty verification command");
        }

        let start = Instant::now();
        let timeout = Duration::from_secs(self.config.timeout_secs);
        
        let shell = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        let mut child = Command::new(shell.0)
            .arg(shell.1)
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn verification command")?;

        // Enforce Timeout logic (Fixes Double Wait & Clippy)
        let status_code = if let Some(status) = child.wait_timeout(timeout).context("Failed to wait")? {
            status.code()
        } else {
            // Timeout occurred, kill the child
            let _ = child.kill();
            // Wait to clean up the zombie process
            let _ = child.wait(); 
            bail!("Verification timed out after {}s", self.config.timeout_secs);
        };

        let duration = start.elapsed();
        
        // Manual Output Capture (Fixes Double Wait Bug)
        let mut stdout_str = String::new();
        let mut stderr_str = String::new();

        if let Some(mut out) = child.stdout {
            let _ = out.read_to_string(&mut stdout_str);
        }
        if let Some(mut err) = child.stderr {
            let _ = err.read_to_string(&mut stderr_str);
        }

        Ok(VerifyResult {
            success: status_code == Some(0),
            exit_code: status_code,
            stdout: stdout_str,
            stderr: stderr_str,
            duration,
        })
    }

    /// Runs verification with user-friendly output on failure.
    ///
    /// # Errors
    /// Returns error if command fails to execute or times out.
    pub fn verify(&self, cmd: &str) -> Result<VerifyResult> {
        let result = self.run(cmd)?;

        if !result.passed() {
            eprintln!("? Verification Failed ");
            eprintln!(" Command: {cmd}");
            if let Some(code) = result.exit_code {
                eprintln!(" Exit Code: {code}");
            }
            if !result.stderr.is_empty() {
                eprintln!(" Stderr:");
                for line in result.stderr.lines().take(10) {
                    eprintln!("   {line}");
                }
            }
            eprintln!("?");
        }

        Ok(result)
    }
}