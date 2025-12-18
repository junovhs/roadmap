//! Derived State Engine: Utilities for task state.

use super::context::RepoContext;
use super::types::{DerivedStatus, Task};

/// A task with its derived state pre-computed.
#[derive(Debug, Clone)]
pub struct TaskWithState {
    pub task: Task,
    pub state: DerivedStatus,
}

impl TaskWithState {
    #[must_use]
    pub fn new(task: Task, context: &RepoContext) -> Self {
        let state = task.derive_status(context);
        Self { task, state }
    }
}

/// Batch-derives state for multiple tasks.
#[must_use]
pub fn derive_all_states(tasks: Vec<Task>, context: &RepoContext) -> Vec<TaskWithState> {
    tasks
        .into_iter()
        .map(|task| TaskWithState::new(task, context))
        .collect()
}