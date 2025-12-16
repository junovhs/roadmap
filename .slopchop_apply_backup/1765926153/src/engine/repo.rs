//! Task Repository: All database operations in one place.

use super::types::{Proof, Task, TaskStatus};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::ops::Deref;

/// SQL fragments - single source of truth
const TASK_SELECT: &str =
    "SELECT id, slug, title, status, test_cmd, created_at, proof_json FROM tasks";

/// Repository for task operations.
/// Works with both Connection and Transaction (via Deref).
pub struct TaskRepo<C: Deref<Target = Connection>> {
    conn: C,
}

impl<C: Deref<Target = Connection>> TaskRepo<C> {
    #[must_use]
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    /// Returns a reference to the underlying connection.
    #[must_use]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Adds a new task, optionally with a test command.
    ///
    /// # Errors
    /// Returns error if the INSERT fails.
    pub fn add(&self, slug: &str, title: &str, test_cmd: Option<&str>) -> Result<i64> {
        match test_cmd {
            Some(cmd) => {
                self.conn.execute(
                    "INSERT INTO tasks (slug, title, status, test_cmd) VALUES (?1, ?2, ?3, ?4)",
                    params![slug, title, TaskStatus::Pending.to_string(), cmd],
                )?;
            }
            None => {
                self.conn.execute(
                    "INSERT INTO tasks (slug, title, status) VALUES (?1, ?2, ?3)",
                    params![slug, title, TaskStatus::Pending.to_string()],
                )?;
            }
        }
        Ok(self.conn.last_insert_rowid())
    }

    /// Links two tasks.
    ///
    /// # Errors
    /// Returns error if the INSERT fails.
    pub fn link(&self, from_id: i64, to_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO dependencies (blocker_id, blocked_id) VALUES (?1, ?2)",
            params![from_id, to_id],
        )?;
        Ok(())
    }

    /// Retrieves all tasks.
    ///
    /// # Errors
    /// Returns error if the SELECT fails.
    pub fn get_all(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(TASK_SELECT)?;
        let rows = stmt.query_map([], row_to_task)?;

        let mut tasks = Vec::new();
        for task in rows {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    /// Finds a task by its slug.
    ///
    /// # Errors
    /// Returns error if the query fails.
    pub fn find_by_slug(&self, slug: &str) -> Result<Option<Task>> {
        let sql = format!("{TASK_SELECT} WHERE slug = ?1");
        self.conn
            .query_row(&sql, params![slug], row_to_task)
            .optional()
            .context("Failed to find task by slug")
    }

    /// Finds a task by slug (case-insensitive).
    ///
    /// # Errors
    /// Returns error if the query fails.
    pub fn find_by_slug_ci(&self, slug: &str) -> Result<Option<Task>> {
        let sql = format!("{TASK_SELECT} WHERE LOWER(slug) = LOWER(?1)");
        self.conn
            .query_row(&sql, params![slug], row_to_task)
            .optional()
            .context("Failed to find task by slug")
    }

    /// Finds a task by its ID.
    ///
    /// # Errors
    /// Returns error if the query fails.
    pub fn find_by_id(&self, id: i64) -> Result<Option<Task>> {
        let sql = format!("{TASK_SELECT} WHERE id = ?1");
        self.conn
            .query_row(&sql, params![id], row_to_task)
            .optional()
            .context("Failed to find task by id")
    }

    /// Updates the status of a task.
    ///
    /// # Errors
    /// Returns error if the UPDATE fails.
    pub fn update_status(&self, id: i64, status: TaskStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            params![status.to_string(), id],
        )?;
        Ok(())
    }

    /// Saves proof evidence for a task.
    ///
    /// # Errors
    /// Returns error if the UPDATE fails.
    pub fn save_proof(&self, id: i64, proof: &Proof) -> Result<()> {
        let json = serde_json::to_string(proof)?;
        self.conn.execute(
            "UPDATE tasks SET proof_json = ?1 WHERE id = ?2",
            params![json, id],
        )?;
        Ok(())
    }

    /// Sets the active task in the state table.
    ///
    /// # Errors
    /// Returns error if the INSERT/UPDATE fails.
    pub fn set_active_task(&self, task_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO state (key, value) VALUES ('active_task', ?1)",
            params![task_id.to_string()],
        )?;
        Ok(())
    }

    /// Gets the currently active task ID.
    ///
    /// # Errors
    /// Returns error if the query fails.
    pub fn get_active_task_id(&self) -> Result<Option<i64>> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM state WHERE key = 'active_task'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match result {
            Some(s) => Ok(s.parse::<i64>().ok()),
            None => Ok(None),
        }
    }
}

/// Converts a database row to a Task.
pub fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    let status_str: String = row.get(3)?;
    let proof_json: Option<String> = row.get(6)?;
    let proof = proof_json.and_then(|j| serde_json::from_str(&j).ok());

    Ok(Task {
        id: row.get(0)?,
        slug: row.get(1)?,
        title: row.get(2)?,
        status: TaskStatus::from(status_str),
        test_cmd: row.get(4)?,
        created_at: row.get(5)?,
        proof,
    })
}