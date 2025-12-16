//! Derived State Engine: Computes task status from proof evidence.
//!
//! This module is the "truth oracle" - it answers "what is the actual state
//! of this task right now?" by examining proof evidence against current HEAD.
//!
//! Designed to be lightweight and fast for UI consumption (Dioxus dashboard).

use super::types::{Proof, Task, TaskStatus};
use std::process::Command;

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
        matches!(self, DerivedStatus::Unproven | DerivedStatus::Stale | DerivedStatus::Broken)
    }

    /// Returns true if this task satisfies dependency requirements.
    ///
    /// Note: ATTESTED does NOT satisfy dependencies by default.
    /// This is intentional - agents shouldn't be able to "force their way out."
    #[must_use]
    pub fn satisfies_dependency(&self) -> bool {
        matches!(self, DerivedStatus::Proven)
    }

    /// Returns true if this task satisfies dependencies (including attested).
    ///
    /// Use this only when explicitly allowing attested tasks to unblock work.
    #[must_use]
    pub fn satisfies_dependency_lenient(&self) -> bool {
        matches!(self, DerivedStatus::Proven | DerivedStatus::Attested)
    }
}

impl std::fmt::Display for DerivedStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DerivedStatus::Unproven => write!(f, "UNPROVEN"),
            DerivedStatus::Proven => write!(f, "PROVEN"),
            DerivedStatus::Stale => write!(f, "STALE"),
            DerivedStatus::Broken => write!(f, "BROKEN"),
            DerivedStatus::Attested => write!(f, "ATTESTED"),
        }
    }
}

/// Derives the current state of a task based on proof evidence and HEAD.
///
/// This is a pure function - no I/O, no side effects. Fast enough for
/// real-time UI updates (< 1µs per call).
///
/// # Arguments
/// * `task` - The task to evaluate
/// * `head_sha` - Current git HEAD (call `get_head_sha()` once per batch)
#[must_use]
pub fn derive_state(task: &Task, head_sha: &str) -> DerivedStatus {
    // Check for attestation first (stored status indicates manual override)
    if task.status == TaskStatus::Attested {
        return DerivedStatus::Attested;
    }

    // No proof = unproven
    let Some(proof) = &task.proof else {
        return DerivedStatus::Unproven;
    };

    // Proof with non-zero exit = broken
    if proof.exit_code != 0 {
        return DerivedStatus::Broken;
    }

    // Proof passed but HEAD moved = stale
    if !sha_matches(&proof.git_sha, head_sha) {
        return DerivedStatus::Stale;
    }

    // Proof passed and HEAD matches = proven
    DerivedStatus::Proven
}

/// Compare git SHAs (handles short vs full, and "unknown")
fn sha_matches(stored: &str, current: &str) -> bool {
    if stored == "unknown" || current == "unknown" {
        return true; // Be lenient if git unavailable
    }
    let min_len = stored.len().min(current.len()).min(7);
    if min_len == 0 {
        return false;
    }
    stored[..min_len] == current[..min_len]
}

/// Gets the current git HEAD SHA.
///
/// Returns "unknown" if not in a git repository.
/// Cache this value when processing multiple tasks.
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
        let state = derive_state(&task, head_sha);
        Self { task, state }
    }
}

/// Batch-derives state for multiple tasks.
///
/// More efficient than calling `derive_state` in a loop because
/// it fetches HEAD once. Use this for list/dashboard views.
#[must_use]
pub fn derive_all_states(tasks: Vec<Task>) -> Vec<TaskWithState> {
    let head_sha = get_head_sha();
    tasks
        .into_iter()
        .map(|task| TaskWithState::new(task, &head_sha))
        .collect()
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
    fn test_unproven_no_proof() {
        let task = make_task(TaskStatus::Pending, None);
        assert_eq!(derive_state(&task, "abc123"), DerivedStatus::Unproven);
    }

    #[test]
    fn test_proven_matching_sha() {
        let proof = Proof::new("echo ok", 0, "abc123", 100);
        let task = make_task(TaskStatus::Done, Some(proof));
        assert_eq!(derive_state(&task, "abc123"), DerivedStatus::Proven);
    }

    #[test]
    fn test_stale_sha_moved() {
        let proof = Proof::new("echo ok", 0, "abc123", 100);
        let task = make_task(TaskStatus::Done, Some(proof));
        assert_eq!(derive_state(&task, "def456"), DerivedStatus::Stale);
    }

    #[test]
    fn test_broken_nonzero_exit() {
        let proof = Proof::new("exit 1", 1, "abc123", 100);
        let task = make_task(TaskStatus::Active, Some(proof));
        assert_eq!(derive_state(&task, "abc123"), DerivedStatus::Broken);
    }

    #[test]
    fn test_attested_status() {
        let proof = Proof::attested("manual review", "abc123");
        let task = make_task(TaskStatus::Attested, Some(proof));
        assert_eq!(derive_state(&task, "abc123"), DerivedStatus::Attested);
    }

    #[test]
    fn test_actionable_states() {
        assert!(DerivedStatus::Unproven.is_actionable());
        assert!(DerivedStatus::Stale.is_actionable());
        assert!(DerivedStatus::Broken.is_actionable());
        assert!(!DerivedStatus::Proven.is_actionable());
        assert!(!DerivedStatus::Attested.is_actionable());
    }

    #[test]
    fn test_dependency_satisfaction() {
        assert!(DerivedStatus::Proven.satisfies_dependency());
        assert!(!DerivedStatus::Attested.satisfies_dependency());
        assert!(!DerivedStatus::Stale.satisfies_dependency());

        // Lenient mode includes attested
        assert!(DerivedStatus::Proven.satisfies_dependency_lenient());
        assert!(DerivedStatus::Attested.satisfies_dependency_lenient());
    }
}
