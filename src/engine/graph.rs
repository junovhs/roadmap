//! Graph Engine: In-memory DAG representation using petgraph.
//!
//! Provides topological sorting, cycle detection, and frontier computation.

use super::repo::row_to_task;
use super::types::{DerivedStatus, Task};
use anyhow::{bail, Result};
use petgraph::algo::is_cyclic_directed;
use petgraph::graphmap::DiGraphMap;
use rusqlite::Connection;
use std::collections::HashMap;

/// In-memory representation of the task dependency graph.
pub struct TaskGraph {
    graph: DiGraphMap<i64, ()>,
    tasks: HashMap<i64, Task>,
    head_sha: String,
}

impl TaskGraph {
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
            tasks: HashMap::new(),
            head_sha: get_git_sha(),
        }
    }

    /// Loads the entire graph from the database into memory.
    ///
    /// # Errors
    /// Returns error if SQL query fails.
    pub fn build(conn: &Connection) -> Result<Self> {
        let mut graph = DiGraphMap::new();
        let mut task_map = HashMap::new();

        let mut stmt = conn.prepare(
            "SELECT id, slug, title, status, test_cmd, created_at, proof_json FROM tasks",
        )?;
        let rows = stmt.query_map([], row_to_task)?;

        for t in rows {
            let task = t?;
            graph.add_node(task.id);
            task_map.insert(task.id, task);
        }

        let mut stmt = conn.prepare("SELECT blocker_id, blocked_id FROM dependencies")?;
        let edge_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;

        for e in edge_rows {
            let (source, target) = e?;
            graph.add_edge(source, target, ());
        }

        Ok(Self {
            graph,
            tasks: task_map,
            head_sha: get_git_sha(),
        })
    }

    /// Checks if adding an edge would create a cycle.
    #[must_use]
    pub fn would_create_cycle(&self, from_id: i64, to_id: i64) -> bool {
        let mut test_graph = self.graph.clone();
        test_graph.add_edge(from_id, to_id, ());
        is_cyclic_directed(&test_graph)
    }

    /// Validates that the graph has no cycles.
    ///
    /// # Errors
    /// Returns error if a cycle is detected.
    pub fn validate(&self) -> Result<()> {
        if is_cyclic_directed(&self.graph) {
            bail!("Cycle detected in task dependencies! A blocks B blocks A.");
        }
        Ok(())
    }

    /// Returns the frontier - tasks that are actionable and unblocked.
    ///
    /// A task is on the frontier if:
    /// 1. Its derived status is UNPROVEN, STALE, or BROKEN
    /// 2. All its blockers are PROVEN or ATTESTED
    #[must_use]
    pub fn get_frontier(&self) -> Vec<&Task> {
        let mut frontier = Vec::new();

        for (id, task) in &self.tasks {
            let status = task.derive_status(&self.head_sha);

            // Skip if already proven or attested
            if matches!(status, DerivedStatus::Proven | DerivedStatus::Attested) {
                continue;
            }

            // Check if blocked by incomplete dependencies
            if !self.is_task_blocked(*id) {
                frontier.push(task);
            }
        }

        // Sort: BROKEN first (needs attention), then STALE, then UNPROVEN
        frontier.sort_by(|a, b| {
            let status_a = a.derive_status(&self.head_sha);
            let status_b = b.derive_status(&self.head_sha);
            let priority = |s: DerivedStatus| match s {
                DerivedStatus::Broken => 0,
                DerivedStatus::Stale => 1,
                DerivedStatus::Unproven => 2,
                _ => 3,
            };
            priority(status_a).cmp(&priority(status_b)).then(a.id.cmp(&b.id))
        });

        frontier
    }

    /// Checks if a task is blocked by incomplete dependencies.
    fn is_task_blocked(&self, task_id: i64) -> bool {
        let blockers =
            self.graph
                .neighbors_directed(task_id, petgraph::Direction::Incoming);

        for source_id in blockers {
            if let Some(parent) = self.tasks.get(&source_id) {
                let parent_status = parent.derive_status(&self.head_sha);
                // Only PROVEN and ATTESTED satisfy dependencies
                if !matches!(
                    parent_status,
                    DerivedStatus::Proven | DerivedStatus::Attested
                ) {
                    return true;
                }
            }
        }
        false
    }

    /// Gets all tasks that are blocked by a given task.
    #[must_use]
    pub fn get_blocked_by(&self, task_id: i64) -> Vec<&Task> {
        self.graph
            .neighbors_directed(task_id, petgraph::Direction::Outgoing)
            .filter_map(|id| self.tasks.get(&id))
            .collect()
    }

    /// Gets all tasks that block a given task.
    #[must_use]
    pub fn get_blockers(&self, task_id: i64) -> Vec<&Task> {
        self.graph
            .neighbors_directed(task_id, petgraph::Direction::Incoming)
            .filter_map(|id| self.tasks.get(&id))
            .collect()
    }

    /// Returns the total number of tasks.
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Returns the total number of edges (dependencies).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Returns the current HEAD SHA used for staleness checks.
    #[must_use]
    pub fn head_sha(&self) -> &str {
        &self.head_sha
    }

    /// Gets a task by ID.
    #[must_use]
    pub fn get_task(&self, id: i64) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// Returns counts of tasks by derived status.
    #[must_use]
    pub fn status_counts(&self) -> StatusCounts {
        let mut counts = StatusCounts::default();
        for task in self.tasks.values() {
            match task.derive_status(&self.head_sha) {
                DerivedStatus::Unproven => counts.unproven += 1,
                DerivedStatus::Proven => counts.proven += 1,
                DerivedStatus::Stale => counts.stale += 1,
                DerivedStatus::Broken => counts.broken += 1,
                DerivedStatus::Attested => counts.attested += 1,
            }
        }
        counts
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct StatusCounts {
    pub unproven: usize,
    pub proven: usize,
    pub stale: usize,
    pub broken: usize,
    pub attested: usize,
}

impl StatusCounts {
    #[must_use]
    pub fn total(&self) -> usize {
        self.unproven + self.proven + self.stale + self.broken + self.attested
    }

    #[must_use]
    pub fn complete(&self) -> usize {
        self.proven + self.attested
    }
}

/// Gets current git HEAD SHA.
fn get_git_sha() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_detection() {
        let mut graph = TaskGraph::new();
        graph.graph.add_node(1);
        graph.graph.add_node(2);
        graph.graph.add_node(3);

        graph.graph.add_edge(1, 2, ());
        graph.graph.add_edge(2, 3, ());

        assert!(graph.validate().is_ok());
        assert!(graph.would_create_cycle(3, 1));
    }
}
