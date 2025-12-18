//! Core types for the Roadmap system.

use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedStatus {
    Unproven,
    Proven,
    Stale,
    Broken,
    Attested,
}

impl DerivedStatus {
    /// Returns true if the task requires attention.
    #[must_use]
    pub fn is_actionable(&self) -> bool {
        matches!(self, Self::Unproven | Self::Stale | Self::Broken)
    }

    /// Returns true if the task fulfills its role as a dependency.
    #[must_use]
    pub fn satisfies_dependency(&self) -> bool {
        matches!(self, Self::Proven | Self::Attested)
    }
}

impl fmt::Display for DerivedStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub status: TaskStatus,
    pub test_cmd: Option<String>,
    pub created_at: String,
    pub proof: Option<Proof>,
}

impl Task {
    /// Computes the derived truth of the task based on proof history and HEAD.
    #[must_use]
    pub fn derive_status(&self, head_sha: &str) -> DerivedStatus {
        let Some(proof) = &self.proof else {
            return DerivedStatus::Unproven;
        };

        if proof.attested_reason.is_some() {
            return DerivedStatus::Attested;
        }

        if proof.exit_code != 0 {
            return DerivedStatus::Broken;
        }

        if !sha_matches(&proof.git_sha, head_sha) {
            return DerivedStatus::Stale;
        }

        DerivedStatus::Proven
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub cmd: String,
    pub exit_code: i32,
    pub git_sha: String,
    pub timestamp: String,
    pub duration_ms: u64,
    pub attested_reason: Option<String>,
}

impl Proof {
    /// Creates a new machine-verified proof.
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

    /// Creates a new human-attested proof.
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

fn sha_matches(stored: &str, current: &str) -> bool {
    if stored == "unknown" || current == "unknown" {
        return true;
    }
    let len = stored.len().min(current.len()).min(7);
    len > 0 && stored[..len] == current[..len]
}