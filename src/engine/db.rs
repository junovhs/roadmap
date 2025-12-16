use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::Path;

const DB_DIR: &str = ".roadmap";
const DB_FILE: &str = "state.db";

pub struct Db;

impl Db {
    /// Initializes the .roadmap directory and `SQLite` database schema.
    ///
    /// # Errors
    /// Returns error if directory creation, DB opening, or migration fails.
    pub fn init() -> Result<()> {
        if !Path::new(DB_DIR).exists() {
            fs::create_dir(DB_DIR).context("Failed to create .roadmap directory")?;
        }

        let db_path = Path::new(DB_DIR).join(DB_FILE);
        let conn = Connection::open(db_path).context("Failed to open database")?;

        Self::migrate(&conn)?;

        Ok(())
    }

    /// Connects to an existing database.
    ///
    /// # Errors
    /// Returns error if the database file does not exist or cannot be opened.
    pub fn connect() -> Result<Connection> {
        let db_path = Path::new(DB_DIR).join(DB_FILE);
        if !db_path.exists() {
            anyhow::bail!("Roadmap not initialized. Run `roadmap init` first.");
        }
        let conn = Connection::open(db_path).context("Failed to open database")?;
        Ok(conn)
    }

    /// Applies the schema migrations.
    fn migrate(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                slug TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                test_cmd TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                context_files TEXT
            )",
            [],
        )
        .context("Failed to create tasks table")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS dependencies (
                blocker_id INTEGER,
                blocked_id INTEGER,
                PRIMARY KEY (blocker_id, blocked_id),
                FOREIGN KEY(blocker_id) REFERENCES tasks(id),
                FOREIGN KEY(blocked_id) REFERENCES tasks(id)
            )",
            [],
        )
        .context("Failed to create dependencies table")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS state (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )
        .context("Failed to create state table")?;

        Ok(())
    }
}
