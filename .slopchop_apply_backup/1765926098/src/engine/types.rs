//! Core types for the Roadmap system.
//!
//! Note: `DerivedStatus` (the computed truth) lives in `state.rs`.
//! `TaskStatus` here is the stored/cached status in SQLite.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stored status in the database.
///
/// Note: This is a cache/legacy field. The **true** status is computed
/// by `state::derive_state()` from proof evidence + current HEAD.
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

/// A task/claim in the roadmap.
#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: i64,
    pub slug: String,
    pub title: String,
    /// Cached status (see `state::derive_state()` for truth)
    pub status: TaskStatus,
    pub test_cmd: Option<String>,
    pub created_at: String,
    pub proof: Option<Proof>,
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
