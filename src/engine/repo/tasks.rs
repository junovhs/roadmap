//! Task Repository: Core Task operations, Scopes, and State.

use super::proofs::ProofRepo;
use crate::engine::types::{Task, TaskStatus};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

pub const TASK_SELECT: &str = "SELECT id, slug, title, status, test_cmd, created_at FROM tasks";

pub struct TaskRepo<'a> {
    conn: &'a Connection,
}

impl<'a> TaskRepo<'a> {
    /// Creates a new repository instance borrowing the connection.
    #[must_use]
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Returns the underlying database connection.
    #[must_use]
    pub fn conn(&self) -> &Connection {
        self.conn
    }

    /// Adds a new task to the database.
    ///
    /// # Errors
    /// Returns an error if the insertion fails.
    pub fn add(&self, slug: &str, title: &str, test_cmd: Option<&str>) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tasks (slug, title, status, test_cmd) VALUES (?1, ?2, ?3, ?4)",
            params![slug, title, TaskStatus::Pending.to_string(), test_cmd],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Associates a file glob scope with a task.
    ///
    /// # Errors
    /// Returns an error if insertion fails.
    pub fn add_scope(&self, task_id: i64, glob: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO task_scopes (task_id, glob) VALUES (?1, ?2)",
            params![task_id, glob],
        )?;
        Ok(())
    }

    /// Creates a dependency link between two tasks.
    ///
    /// # Errors
    /// Returns an error if the link cannot be created.
    pub fn link(&self, from_id: i64, to_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO dependencies (blocker_id, blocked_id) VALUES (?1, ?2)",
            params![from_id, to_id],
        )?;
        Ok(())
    }

    /// Retrieves all tasks from the database.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub fn get_all(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(TASK_SELECT)?;
        let rows = stmt.query_map([], |r| self.row_to_task(r))?;
        let mut tasks = Vec::new();
        for task in rows {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    /// Finds a task by its slug (case-insensitive).
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub fn find_by_slug(&self, slug: &str) -> Result<Option<Task>> {
        let sql = format!("{TASK_SELECT} WHERE LOWER(slug) = LOWER(?1)");
        self.conn
            .query_row(&sql, params![slug], |r| self.row_to_task(r))
            .optional()
            .context("Search by slug failed")
    }

    /// Finds a task by its internal ID.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub fn find_by_id(&self, id: i64) -> Result<Option<Task>> {
        let sql = format!("{TASK_SELECT} WHERE id = ?1");
        self.conn
            .query_row(&sql, params![id], |r| self.row_to_task(r))
            .optional()
            .context("Search by ID failed")
    }

    /// Retrieves scopes associated with a task.
    ///
    /// # Errors
    /// Returns a `rusqlite` error if query logic fails.
    pub fn get_scopes(&self, task_id: i64) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT glob FROM task_scopes WHERE task_id = ?1")?;
        let rows = stmt.query_map(params![task_id], |row| row.get(0))?;

        let mut scopes = Vec::new();
        for r in rows {
            scopes.push(r?);
        }
        Ok(scopes)
    }

    /// Sets the active task in global state.
    ///
    /// # Errors
    /// Returns an error if the state cannot be updated.
    pub fn set_active_task(&self, task_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO state (key, value) VALUES ('active_task', ?1)",
            params![task_id.to_string()],
        )?;
        Ok(())
    }

    /// Retrieves the ID of the currently active task.
    ///
    /// # Errors
    /// Returns an error if the state query fails.
    pub fn get_active_task_id(&self) -> Result<Option<i64>> {
        let res: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM state WHERE key = 'active_task'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(res.and_then(|s| s.parse().ok()))
    }

    /// Updates the cached status column of a task.
    ///
    /// # Errors
    /// Returns an error if the update fails.
    pub fn update_status(&self, id: i64, status: TaskStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            params![status.to_string(), id],
        )?;
        Ok(())
    }

    /// Converts a database row to a Task object.
    ///
    /// # Errors
    /// Returns a `rusqlite` error if data conversion fails.
    pub fn row_to_task(&self, row: &rusqlite::Row) -> rusqlite::Result<Task> {
        let id: i64 = row.get(0)?;
        let proof_repo = ProofRepo::new(self.conn);
        let proof = proof_repo.get_latest(id)?;
        let scopes = self.get_scopes(id)?;

        Ok(Task {
            id,
            slug: row.get(1)?,
            title: row.get(2)?,
            status: TaskStatus::from(row.get::<_, String>(3)?),
            test_cmd: row.get(4)?,
            created_at: row.get(5)?,
            proof,
            scopes,
        })
    }
}