//! Repository Context: The oracle for repo state and file changes.

use anyhow::Result;
use std::process::Command;

/// Encapsulates the state of the git repository.
///
/// Handles caching `git diff` operations to determine scoped staleness efficiently.
pub struct RepoContext {
    pub head_sha: String,
    pub is_dirty: bool,
}

impl RepoContext {
    /// Captures the current repository state.
    ///
    /// # Errors
    /// Returns error if git execution fails.
    pub fn new() -> Result<Self> {
        let head_sha = get_git_sha();
        let is_dirty = check_if_dirty();
        Ok(Self { head_sha, is_dirty })
    }

    /// Returns the current HEAD SHA.
    #[must_use]
    pub fn head_sha(&self) -> &str {
        &self.head_sha
    }

    /// Checks if files matching the given scopes have changed between `since_sha` and HEAD.
    ///
    /// # Returns
    /// - `true` if changes are detected or if git fails (safe default).
    /// - `false` if `git diff --quiet` returns 0 (no changes).
    #[must_use]
    pub fn has_changes(&self, since_sha: &str, scopes: &[String]) -> bool {
        if scopes.is_empty() {
            return true; // No scope implies global sensitivity
        }

        // If SHAs match, obviously no changes.
        if since_sha == self.head_sha {
            return false;
        }

        // git diff --quiet <old> HEAD -- <paths...>
        // Returns 0 if no changes, 1 if changes.
        let mut cmd = Command::new("git");
        cmd.arg("diff")
           .arg("--quiet")
           .arg(since_sha)
           .arg("HEAD")
           .arg("--");
        
        for scope in scopes {
            cmd.arg(scope);
        }

        match cmd.status() {
            Ok(status) => !status.success(), // success (0) means NO changes
            Err(_) => true, // If git fails, assume the worst (Stale)
        }
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

fn check_if_dirty() -> bool {
    // git status --porcelain
    // Prints nothing if clean. Prints lines if dirty.
    // If git command fails, we default to true (safe side).
    match Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
    {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => true,
    }
}