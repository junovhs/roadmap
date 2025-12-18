//! Repository Context: The oracle for repo state and file changes.

use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Command;

/// Encapsulates the state of the git repository.
///
/// Includes a memoization cache to prevent redundant `git diff` calls
/// when multiple tasks share the same scope or proof SHA.
pub struct RepoContext {
    pub head_sha: String,
    pub is_dirty: bool,
    // Memoization: (since_sha + scopes_key) -> bool
    cache: RefCell<HashMap<String, bool>>,
}

impl RepoContext {
    /// Captures the current repository state.
    ///
    /// # Errors
    /// Returns error if git execution fails.
    pub fn new() -> Result<Self> {
        let head_sha = get_git_sha();
        let is_dirty = check_if_dirty();
        Ok(Self { 
            head_sha, 
            is_dirty,
            cache: RefCell::new(HashMap::new()),
        })
    }

    /// Creates a context from a known SHA (useful for read-only views).
    ///
    /// Initializes `is_dirty` to false and an empty cache.
    #[must_use]
    pub fn from_sha(head_sha: String) -> Self {
        Self {
            head_sha,
            is_dirty: false,
            cache: RefCell::new(HashMap::new()),
        }
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
            return true; // Global sensitivity
        }

        if since_sha == self.head_sha {
            return false;
        }

        // Create a unique key for the cache: "sha|scope1|scope2"
        let mut key_parts = vec![since_sha.to_string()];
        key_parts.extend_from_slice(scopes);
        let key = key_parts.join("|");

        // Check Cache
        if let Some(&cached) = self.cache.borrow().get(&key) {
            return cached;
        }

        // Cache Miss: Run Git
        let has_change = Self::run_git_diff(since_sha, scopes);
        
        // Store Result
        self.cache.borrow_mut().insert(key, has_change);
        has_change
    }

    fn run_git_diff(since_sha: &str, scopes: &[String]) -> bool {
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
            Ok(status) => !status.success(), 
            Err(_) => true, 
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
    match Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
    {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => true,
    }
}