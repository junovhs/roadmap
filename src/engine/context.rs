//! Repository Context: The oracle for repo state and file changes.

use anyhow::Result;
use std::process::Command;

/// Encapsulates the state of the git repository.
///
/// In the future (v0.3.0), this will handle caching `git diff` operations
/// to determine scoped staleness efficiently.
pub struct RepoContext {
    pub head_sha: String,
}

impl RepoContext {
    /// Captures the current repository state.
    ///
    /// # Errors
    /// Returns error if git execution fails.
    pub fn new() -> Result<Self> {
        let head_sha = get_git_sha();
        Ok(Self { head_sha })
    }

    /// Returns the current HEAD SHA.
    #[must_use]
    pub fn head_sha(&self) -> &str {
        &self.head_sha
    }
}

fn get_git_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
}