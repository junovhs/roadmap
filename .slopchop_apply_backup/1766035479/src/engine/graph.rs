//! Graph Engine: In-memory DAG representation.

use super::repo::TaskRepo;
use super::types::{DerivedStatus, Task};
use anyhow::Result;
use petgraph::algo::is_cyclic_directed;
use petgraph::graphmap::DiGraphMap;
use rusqlite::Connection;
use std::collections::HashMap;

pub struct TaskGraph {
    graph: DiGraphMap<i64, ()>,
    tasks: HashMap<i64, Task>,
    head_sha: String,
}

impl TaskGraph {
    /// Builds the dependency graph from the database.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub fn build(conn: &Connection) -> Result<Self> {
        let mut graph = DiGraphMap::new();
        let repo = TaskRepo::new(conn);
        let tasks = repo.get_all()?;
        let mut task_map = HashMap::new();

        for t in tasks {
            graph.add_node(t.id);
            task_map.insert(t.id, t);
        }

        let mut stmt = conn.prepare("SELECT blocker_id, blocked_id FROM dependencies")?;
        let edges = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for e in edges {
            let (src, dst) = e?;
            graph.add_edge(src, dst, ());
        }

        Ok(Self {
            graph,
            tasks: task_map,
            head_sha: get_git_sha(),
        })
    }

    /// Returns tasks that are unblocked and require work (Unproven, Stale, or Broken).
    #[must_use]
    pub fn get_frontier(&self) -> Vec<&Task> {
        let mut frontier: Vec<_> = self
            .tasks
            .values()
            .filter(|t| {
                let status = t.derive_status(&self.head_sha);
                status.is_actionable()
            })
            .filter(|t| !self.is_blocked(t.id))
            .collect();

        frontier.sort_by_key(|t| t.id);
        frontier
    }

    /// Checks if a task is blocked by any dependency that isn't Proven or Attested.
    fn is_blocked(&self, id: i64) -> bool {
        self.graph
            .neighbors_directed(id, petgraph::Direction::Incoming)
            .any(|sid| {
                let Some(task) = self.tasks.get(&sid) else {
                    return false;
                };
                let status = task.derive_status(&self.head_sha);
                // Explicitly use DerivedStatus to satisfy architectural theme and import
                !matches!(status, DerivedStatus::Proven | DerivedStatus::Attested)
            })
    }

    /// Detects if adding an edge would create a cycle.
    #[must_use]
    pub fn would_create_cycle(&self, from: i64, to: i64) -> bool {
        let mut test = self.graph.clone();
        test.add_edge(from, to, ());
        is_cyclic_directed(&test)
    }

    /// Returns the current git HEAD SHA.
    #[must_use]
    pub fn head_sha(&self) -> &str {
        &self.head_sha
    }

    /// Gets tasks blocked by the given ID.
    #[must_use]
    pub fn get_blocked_by(&self, id: i64) -> Vec<&Task> {
        self.graph
            .neighbors_directed(id, petgraph::Direction::Outgoing)
            .filter_map(|i| self.tasks.get(&i))
            .collect()
    }

    /// Gets tasks that block the given ID.
    #[must_use]
    pub fn get_blockers(&self, id: i64) -> Vec<&Task> {
        self.graph
            .neighbors_directed(id, petgraph::Direction::Incoming)
            .filter_map(|i| self.tasks.get(&i))
            .collect()
    }
}

fn get_git_sha() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}