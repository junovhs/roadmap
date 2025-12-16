//! Derived State Engine: Utilities for task state.
//!
//! Note: Core status logic is now in `types.rs`.

use super::types::{DerivedStatus, Task};
use std::process::Command;

/// Gets the current git HEAD SHA.
///
/// Returns "unknown" if not in a git repository.
#[must_use]
pub fn get_head_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
}

/// A task with its derived state pre-computed.
///
/// Useful for UI rendering where you need both task data and state.
#[derive(Debug, Clone)]
pub struct TaskWithState {
    pub task: Task,
    pub state: DerivedStatus,
}

impl TaskWithState {
    #[must_use]
    pub fn new(task: Task, head_sha: &str) -> Self {
        let state = task.derive_status(head_sha);
        Self { task, state }
    }
}

/// Batch-derives state for multiple tasks.
#[must_use]
pub fn derive_all_states(tasks: Vec<Task>) -> Vec<TaskWithState> {
    let head_sha = get_head_sha();
    tasks
        .into_iter()
        .map(|task| TaskWithState::new(task, &head_sha))
        .collect()
}