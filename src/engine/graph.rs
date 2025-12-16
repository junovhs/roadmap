use super::types::{Task, TaskStatus};
use anyhow::{bail, Result};
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graphmap::DiGraphMap;
use rusqlite::Connection;
use std::collections::HashMap;

pub struct TaskGraph {
    graph: DiGraphMap<i64, ()>,
    tasks: HashMap<i64, Task>,
}

impl TaskGraph {
    /// Loads the entire graph from the database into memory.
    pub fn build(conn: &Connection) -> Result<Self> {
        let mut graph = DiGraphMap::new();
        let mut task_map = HashMap::new();

        // 1. Load Nodes
        let mut stmt = conn.prepare("SELECT id, slug, title, status, test_cmd, created_at FROM tasks")?;
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

        // 2. Load Edges
        let mut stmt = conn.prepare("SELECT blocker_id, blocked_id FROM dependencies")?;
        let edge_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;

        for e in edge_rows {
            let (blocker, blocked) = e?;
            graph.add_edge(blocker, blocked, ());
        }

        Ok(Self { graph, tasks: task_map })
    }

    /// Checks if the graph is valid (no cycles).
    pub fn validate(&self) -> Result<()> {
        if is_cyclic_directed(&self.graph) {
            bail!("Cycle detected in task dependencies! A blocks B blocks A.");
        }
        Ok(())
    }

    /// Returns the "Next" actionable tasks (Topological sort filtered by status).
    pub fn get_critical_path(&self) -> Vec<&Task> {
        // Simple logic:
        // 1. Filter out DONE tasks.
        // 2. Find tasks with in_degree 0 (ignoring DONE dependencies).
        
        let mut actionable = Vec::new();

        for (id, task) in &self.tasks {
            if task.status == TaskStatus::Done {
                continue;
            }

            // Check if all blockers are DONE
            let blockers = self.graph.neighbors_directed(*id, petgraph::Direction::Incoming);
            let mut is_blocked = false;
            for blocker_id in blockers {
                if let Some(parent) = self.tasks.get(&blocker_id) {
                    if parent.status != TaskStatus::Done {
                        is_blocked = true;
                        break;
                    }
                }
            }

            if !is_blocked {
                actionable.push(task);
            }
        }

        // Sort by ID to keep it deterministic for now (could be Priority later)
        actionable.sort_by_key(|t| t.id);
        actionable
    }
}