use super::types::{Task, TaskStatus};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

pub struct TaskRepo {
    conn: Connection,
}

impl TaskRepo {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn add(&self, slug: &str, title: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tasks (slug, title, status) VALUES (?1, ?2, ?3)",
            params![slug, title, TaskStatus::Pending.to_string()],
        )
        .context("Failed to insert task")?;
        
        let id = self.conn.last_insert_rowid();
        Ok(id)
    }

    pub fn link(&self, blocker_id: i64, blocked_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO dependencies (blocker_id, blocked_id) VALUES (?1, ?2)",
            params![blocker_id, blocked_id],
        )?;
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare("SELECT id, slug, title, status, test_cmd, created_at FROM tasks")?;
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

        let mut tasks = Vec::new();
        for task in rows {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn find_by_slug(&self, slug: &str) -> Result<Option<Task>> {
        self.conn.query_row(
            "SELECT id, slug, title, status, test_cmd, created_at FROM tasks WHERE slug = ?1",
            params![slug],
            |row| {
                let status_str: String = row.get(3)?;
                Ok(Task {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    title: row.get(2)?,
                    status: TaskStatus::from(status_str),
                    test_cmd: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .optional()
        .context("Failed to find task by slug")
    }
}