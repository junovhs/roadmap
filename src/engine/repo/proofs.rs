//! Proof Repository: Handles verification evidence and audit logs.

use crate::engine::types::Proof;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

pub struct ProofRepo<'a> {
    conn: &'a Connection,
}

impl<'a> ProofRepo<'a> {
    /// Creates a new proof repository instance.
    #[must_use]
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Records a verification proof for a task.
    ///
    /// # Errors
    /// Returns an error if the proof cannot be saved.
    pub fn save(&self, task_id: i64, proof: &Proof) -> Result<()> {
        self.conn.execute(
            "INSERT INTO proofs (task_id, cmd, exit_code, git_sha, duration_ms, attested_reason, stdout, stderr) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                task_id,
                proof.cmd,
                proof.exit_code,
                proof.git_sha,
                proof.duration_ms,
                proof.attested_reason,
                proof.stdout,
                proof.stderr
            ],
        )?;
        Ok(())
    }

    /// Gets the most recent proof recorded for a task.
    ///
    /// # Errors
    /// Returns a `rusqlite` error if query logic fails.
    pub fn get_latest(&self, task_id: i64) -> rusqlite::Result<Option<Proof>> {
        self.conn
            .query_row(
                "SELECT cmd, exit_code, git_sha, duration_ms, timestamp, attested_reason, stdout, stderr 
                 FROM proofs WHERE task_id = ?1 ORDER BY timestamp DESC LIMIT 1",
                params![task_id],
                |row| {
                    Ok(Proof {
                        cmd: row.get(0)?,
                        exit_code: row.get(1)?,
                        git_sha: row.get(2)?,
                        duration_ms: row.get(3)?,
                        timestamp: row.get(4)?,
                        attested_reason: row.get(5)?,
                        stdout: row.get(6)?,
                        stderr: row.get(7)?,
                    })
                },
            )
            .optional()
    }

    /// Retrieves the full history of proofs for a task.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub fn get_history(&self, task_id: i64) -> Result<Vec<Proof>> {
        let mut stmt = self.conn.prepare(
            "SELECT cmd, exit_code, git_sha, duration_ms, timestamp, attested_reason, stdout, stderr 
             FROM proofs WHERE task_id = ?1 ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map(params![task_id], |row| {
            Ok(Proof {
                cmd: row.get(0)?,
                exit_code: row.get(1)?,
                git_sha: row.get(2)?,
                duration_ms: row.get(3)?,
                timestamp: row.get(4)?,
                attested_reason: row.get(5)?,
                stdout: row.get(6)?,
                stderr: row.get(7)?,
            })
        })?;

        let mut proofs = Vec::new();
        for p in rows {
            proofs.push(p?);
        }
        Ok(proofs)
    }

    /// Retrieves global proof history joined with task slugs.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub fn get_global_history(&self, limit: usize) -> Result<Vec<(String, Proof)>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.slug, p.cmd, p.exit_code, p.git_sha, p.duration_ms, p.timestamp, p.attested_reason, p.stdout, p.stderr 
             FROM proofs p 
             JOIN tasks t ON p.task_id = t.id 
             ORDER BY p.timestamp DESC 
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            let slug: String = row.get(0)?;
            let proof = Proof {
                cmd: row.get(1)?,
                exit_code: row.get(2)?,
                git_sha: row.get(3)?,
                duration_ms: row.get(4)?,
                timestamp: row.get(5)?,
                attested_reason: row.get(6)?,
                stdout: row.get(7)?,
                stderr: row.get(8)?,
            };
            Ok((slug, proof))
        })?;

        let mut history = Vec::new();
        for item in rows {
            history.push(item?);
        }
        Ok(history)
    }
}