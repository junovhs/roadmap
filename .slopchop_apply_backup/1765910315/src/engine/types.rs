use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TaskStatus {
    Pending,
    Active,
    Done,
    Blocked,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "PENDING"),
            TaskStatus::Active => write!(f, "ACTIVE"),
            TaskStatus::Done => write!(f, "DONE"),
            TaskStatus::Blocked => write!(f, "BLOCKED"),
        }
    }
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ACTIVE" => TaskStatus::Active,
            "DONE" => TaskStatus::Done,
            "BLOCKED" => TaskStatus::Blocked,
            _ => TaskStatus::Pending,
        }
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

/// Evidence that a task was verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub cmd: String,
    pub exit_code: i32,
    pub git_sha: String,
    pub timestamp: String,
    pub duration_ms: u64,
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
        }
    }
}