//! Core types for the Roadmap system.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stored status in the database.
///
/// Note: This is a cache/legacy field. The **true** status is computed
/// by `Task::derive_status()` from proof evidence + current HEAD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Active,
    Done,
    Blocked,
    Attested,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Done => write!(f, "DONE"),
            Self::Blocked => write!(f, "BLOCKED"),
            Self::Attested => write!(f, "ATTESTED"),
        }
    }
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ACTIVE" => Self::Active,
            "DONE" => Self::Done,
            "BLOCKED" => Self::Blocked,
            "ATTESTED" => Self::Attested,
            _ => Self::Pending,
        }
    }
}

/// The derived (computed) state of a task.
///
/// Unlike `TaskStatus` (which is stored), `DerivedStatus` is computed
/// from proof evidence and current repository state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedStatus {
    /// No proof exists - task has never been verified
    Unproven,
    /// Proof passed and is still valid for current HEAD
    Proven,
    /// Proof passed, but HEAD has moved since verification
    Stale,
    /// Proof ran and failed (exit code != 0)
    Broken,
    /// Manually attested (human override, not machine-verified)
    Attested,
}

impl DerivedStatus {
    /// Returns the display color hint for UI rendering.
    #[must_use]
    pub fn color_hint(&self) -> &'static str {
        match self {
            DerivedStatus::Proven => "green",
            DerivedStatus::Stale => "amber",
            DerivedStatus::Broken => "red",
            DerivedStatus::Unproven => "gray",
            DerivedStatus::Attested => "blue",
        }
    }

    /// Returns true if this task should appear in the frontier (actionable).
    #[must_use]
    pub fn is_actionable(&self) -> bool {
        matches!(
            self,
            DerivedStatus::Unproven | DerivedStatus::Stale | DerivedStatus::Broken
        )
    }

    /// Returns true if this task satisfies dependency requirements.
    #[must_use]
    pub fn satisfies_dependency(&self) -> bool {
        matches!(self, DerivedStatus::Proven)
    }

    /// Returns true if this task satisfies dependencies (including attested).
    #[must_use]
    pub fn satisfies_dependency_lenient(&self) -> bool {
        matches!(self, DerivedStatus::Proven | DerivedStatus::Attested)
    }
}

impl fmt::Display for DerivedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DerivedStatus::Unproven => write!(f, "UNPROVEN"),
            DerivedStatus::Proven => write!(f, "PROVEN"),
            DerivedStatus::Stale => write!(f, "STALE"),
            DerivedStatus::Broken => write!(f, "BROKEN"),
            DerivedStatus::Attested => write!(f, "ATTESTED"),
        }
    }
}

/// A task/claim in the roadmap.
#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: i64,
    pub slug: String,
    pub title: String,
    /// Cached status (see `derive_status()` for truth)
    pub status: TaskStatus,
    pub test_cmd: Option<String>,
    pub created_at: String,
    pub proof: Option<Proof>,
}

impl Task {
    /// Derives the current state of a task based on proof evidence and HEAD.
    #[must_use]
    pub fn derive_status(&self, head_sha: &str) -> DerivedStatus {
        if self.status == TaskStatus::Attested {
            return DerivedStatus::Attested;
        }

        let Some(proof) = &self.proof else {
            return DerivedStatus::Unproven;
        };

        if proof.exit_code != 0 {
            return DerivedStatus::Broken;
        }

        if !sha_matches(&proof.git_sha, head_sha) {
            return DerivedStatus::Stale;
        }

        DerivedStatus::Proven
    }
}

/// Evidence that a task was verified or attested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub cmd: String,
    pub exit_code: i32,
    pub git_sha: String,
    pub timestamp: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attested_reason: Option<String>,
}

impl Proof {
    #[must_use]
    pub fn new(cmd: &str, exit_code: i32, git_sha: &str, duration_ms: u64) -> Self {
        Self {
            cmd: cmd.to_string(),
            exit_code,
            git_sha: git_sha.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms,
            attested_reason: None,
        }
    }

    #[must_use]
    pub fn attested(reason: &str, git_sha: &str) -> Self {
        Self {
            cmd: "--force".to_string(),
            exit_code: 0,
            git_sha: git_sha.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: 0,
            attested_reason: Some(reason.to_string()),
        }
    }
}

/// Compare git SHAs (handles short vs full, and "unknown")
fn sha_matches(stored: &str, current: &str) -> bool {
    if stored == "unknown" || current == "unknown" {
        return true;
    }
    let min_len = stored.len().min(current.len()).min(7);
    if min_len == 0 {
        return false;
    }
    stored[..min_len] == current[..min_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(status: TaskStatus, proof: Option<Proof>) -> Task {
        Task {
            id: 1,
            slug: "test".to_string(),
            title: "Test Task".to_string(),
            status,
            test_cmd: Some("echo ok".to_string()),
            created_at: "2024-01-01".to_string(),
            proof,
        }
    }

    #[test]
    fn test_derive_status() {
        let t1 = make_task(TaskStatus::Pending, None);
        assert_eq!(t1.derive_status("abc"), DerivedStatus::Unproven);

        let p_ok = Proof::new("cmd", 0, "abc", 100);
        let t2 = make_task(TaskStatus::Done, Some(p_ok));
        assert_eq!(t2.derive_status("abc"), DerivedStatus::Proven);
        assert_eq!(t2.derive_status("xyz"), DerivedStatus::Stale);
    }
}