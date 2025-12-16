//! Fuzzy Task Resolver: Matches human-friendly names to task IDs.

use super::fuzzy::calculate_score;
use super::types::{Task, TaskStatus};
use anyhow::{bail, Result};
use rusqlite::Connection;

pub use super::fuzzy::slugify;

/// Result of a fuzzy resolution attempt.
#[derive(Debug)]
pub struct ResolveResult {
    pub task: Task,
    pub confidence: f64,
    pub match_type: MatchType,
}

#[derive(Debug, PartialEq)]
pub enum MatchType {
    ExactId,
    ExactSlug,
    FuzzyMatch,
}

/// Resolves a query string to a task.
pub struct TaskResolver<'a> {
    conn: &'a Connection,
}

impl<'a> TaskResolver<'a> {
    #[must_use]
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Resolves a query to a task with confidence scoring.
    ///
    /// # Errors
    /// Returns error if no task matches or query is ambiguous.
    pub fn resolve(&self, query: &str) -> Result<ResolveResult> {
        let query = query.trim();

        if let Ok(id) = query.parse::<i64>() {
            if let Some(task) = self.find_by_id(id)? {
                return Ok(ResolveResult { task, confidence: 1.0, match_type: MatchType::ExactId });
            }
        }

        if let Some(task) = self.find_by_slug(query)? {
            return Ok(ResolveResult { task, confidence: 1.0, match_type: MatchType::ExactSlug });
        }

        self.fuzzy_resolve(query)
    }

    fn fuzzy_resolve(&self, query: &str) -> Result<ResolveResult> {
        let candidates = self.fuzzy_search(query)?;

        if candidates.is_empty() {
            bail!("No task found matching '{query}'");
        }

        let best = &candidates[0];
        if best.confidence < 0.4 {
            let suggestions = format_suggestions(&candidates);
            bail!("Ambiguous query '{query}'. Did you mean:\n{suggestions}");
        }

        Ok(ResolveResult {
            task: best.task.clone(),
            confidence: best.confidence,
            match_type: MatchType::FuzzyMatch,
        })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Task>> {
        let sql = "SELECT id, slug, title, status, test_cmd, created_at FROM tasks WHERE id = ?1";
        query_task_by_id(self.conn, sql, id)
    }

    fn find_by_slug(&self, slug: &str) -> Result<Option<Task>> {
        let sql = "SELECT id, slug, title, status, test_cmd, created_at FROM tasks WHERE LOWER(slug) = LOWER(?1)";
        query_task_by_str(self.conn, sql, slug)
    }

    fn fuzzy_search(&self, query: &str) -> Result<Vec<ResolveResult>> {
        let tasks = load_all_tasks(self.conn)?;
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<ResolveResult> = tasks
            .into_iter()
            .map(|task| {
                let score = calculate_score(&task, &query_lower, &query_words);
                ResolveResult { task, confidence: score, match_type: MatchType::FuzzyMatch }
            })
            .filter(|r| r.confidence > 0.0)
            .collect();

        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }
}

fn format_suggestions(candidates: &[ResolveResult]) -> String {
    candidates.iter().take(3)
        .map(|c| format!("  - [{}] {}", c.task.slug, c.task.title))
        .collect::<Vec<_>>()
        .join("\n")
}

fn query_task_by_id(conn: &Connection, sql: &str, id: i64) -> Result<Option<Task>> {
    match conn.query_row(sql, [id], row_to_task) {
        Ok(task) => Ok(Some(task)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn query_task_by_str(conn: &Connection, sql: &str, s: &str) -> Result<Option<Task>> {
    match conn.query_row(sql, [s], row_to_task) {
        Ok(task) => Ok(Some(task)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    let status_str: String = row.get(3)?;
    Ok(Task {
        id: row.get(0)?,
        slug: row.get(1)?,
        title: row.get(2)?,
        status: TaskStatus::from(status_str),
        test_cmd: row.get(4)?,
        created_at: row.get(5)?,
        proof: None,
    })
}

fn load_all_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT id, slug, title, status, test_cmd, created_at FROM tasks")?;
    let rows = stmt.query_map([], row_to_task)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}