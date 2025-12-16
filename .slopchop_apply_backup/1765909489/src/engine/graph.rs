//! Graph Engine: In-memory DAG representation using petgraph.
//!
//! Provides topological sorting and cycle detection for task dependencies.

use super::types::{Task, TaskStatus};
use anyhow::{bail, Result};
use petgraph::algo::is_cyclic_directed;
use petgraph::graphmap::DiGraphMap;
use rusqlite::Connection;
use std::collections::HashMap;

/// In-memory representation of the task dependency graph.
pub struct TaskGraph {
    graph: DiGraphMap<i64, ()>,
    tasks: HashMap<i64, Task>,
}

impl TaskGraph {
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Loads the entire graph from the database into memory.
    ///
    /// # Errors
    /// Returns error if SQL query fails.
    pub fn build(conn: &Connection) -> Result<Self> {
        let mut graph = DiGraphMap::new();
        let mut task_map = HashMap::new();

        // 1. Load Nodes
        let mut stmt = conn.prepare(
            "SELECT id, slug, title, status, test_cmd, created_at FROM tasks"
        )?;
        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(3)?;
            Ok(Task {
                id: row.get(0)?,
                slug: row.get(1)?,
                title: row.get(2)?,
                status: TaskStatus::from(status_str),
                test_cmd: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        for t in rows {
            let task = t?;
            graph.add_node(task.id);
            task_map.insert(task.id, task);
        }

        // 2. Load Edges (blocker -> blocked)
        let mut stmt = conn.prepare("SELECT blocker_id, blocked_id FROM dependencies")?;
        let edge_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;

        for e in edge_rows {
            let (source, target) = e?;
            graph.add_edge(source, target, ());
        }

        Ok(Self { graph, tasks: task_map })
    }

    /// Checks if adding an edge would create a cycle.
    ///
    /// Uses a temporary graph to test acyclicity before commit.
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

    /// Returns the frontier - tasks that are unproven and unblocked.
    ///
    /// A task is on the frontier if:
    /// 1. It is not DONE
    /// 2. All its blockers are DONE (`in_degree` of active blockers == 0)
    #[must_use]
    pub fn get_frontier(&self) -> Vec<&Task> {
        let mut frontier = Vec::new();

        for (id, task) in &self.tasks {
            if task.status == TaskStatus::Done {
                continue;
            }

            if !self.is_task_blocked(*id) {
                frontier.push(task);
            }
        }

        // Sort by ID to keep it deterministic
        frontier.sort_by_key(|t| t.id);
        frontier
    }

    /// Checks if a task is blocked by any incomplete dependencies.
    fn is_task_blocked(&self, task_id: i64) -> bool {
        let blockers = self.graph.neighbors_directed(
            task_id,
            petgraph::Direction::Incoming,
        );

        for source_id in blockers {
            if let Some(parent) = self.tasks.get(&source_id) {
                if parent.status != TaskStatus::Done {
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
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
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

        // 1 -> 2 -> 3 (no cycle)
        graph.graph.add_edge(1, 2, ());
        graph.graph.add_edge(2, 3, ());

        assert!(graph.validate().is_ok());

        // Adding 3 -> 1 would create a cycle
        assert!(graph.would_create_cycle(3, 1));
    }
}