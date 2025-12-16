use serde::Serialize;
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
}
