//! Fuzzy Task Resolver: Matches human queries to Task IDs.

use super::repo::{TaskRepo, TASK_SELECT};
use super::types::Task;
use anyhow::{bail, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;

pub struct ResolveResult {
    pub task: Task,
    pub confidence: f64,
}

pub struct TaskResolver<'a> {
    repo: TaskRepo<'a>,
    conn: &'a Connection,
    strict: bool,
}

impl<'a> TaskResolver<'a> {
    /// Creates a new resolver.
    #[must_use]
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            repo: TaskRepo::new(conn),
            conn,
            strict: false,
        }
    }

    /// Creates a resolver in strict mode.
    #[must_use]
    pub fn strict(conn: &'a Connection) -> Self {
        Self {
            repo: TaskRepo::new(conn),
            conn,
            strict: true,
        }
    }

    /// Resolves a user query into a task.
    ///
    /// # Errors
    /// Returns an error if no match is found or the query is ambiguous.
    pub fn resolve(&self, query: &str) -> Result<ResolveResult> {
        if let Ok(id) = query.parse::<i64>() {
            if let Some(task) = self.repo.find_by_id(id)? {
                return Ok(ResolveResult {
                    task,
                    confidence: 1.0,
                });
            }
        }

        let sql = format!("{TASK_SELECT} WHERE LOWER(slug) = LOWER(?1)");
        let exact: Option<Task> = self
            .conn
            .query_row(&sql, params![query], |r| self.repo.row_to_task(r))
            .optional()?;

        if let Some(task) = exact {
            return Ok(ResolveResult {
                task,
                confidence: 1.0,
            });
        }

        if self.strict {
            bail!("No exact match for '{query}' in strict mode.");
        }
        self.fuzzy_resolve(query)
    }

    fn fuzzy_resolve(&self, query: &str) -> Result<ResolveResult> {
        let tasks = self.repo.get_all()?;
        let query_lower = query.to_lowercase();
        let words: Vec<_> = query_lower.split_whitespace().collect();

        let mut matches: Vec<_> = tasks
            .into_iter()
            .map(|t| (calculate_score(&t, &query_lower, &words), t))
            .filter(|(s, _)| *s > 0.3)
            .collect();

        matches.sort_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
        });

        let (_, task) = matches
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No task matches '{query}'"))?;

        Ok(ResolveResult {
            task,
            confidence: 1.0,
        })
    }
}

/// Generates a slug from a title string.
#[must_use]
pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Calculates a match score between a task and a query.
fn calculate_score(task: &Task, query: &str, query_words: &[&str]) -> f64 {
    let slug_lower = task.slug.to_lowercase();
    let title_lower = task.title.to_lowercase();

    let mut score = 0.0;

    if slug_lower.contains(query) {
        score += 0.8;
    }
    if title_lower.contains(query) {
        score += 0.7;
    }

    for word in query_words {
        if slug_lower.contains(word) {
            score += 0.3;
        }
        if title_lower.contains(word) {
            score += 0.25;
        }
    }

    if slug_lower.starts_with(query) {
        score += 0.5;
    }

    let slug_sim = string_similarity(&slug_lower, query);
    score += slug_sim * 0.4;

    score.min(1.0)
}

#[allow(clippy::cast_precision_loss)]
fn string_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let a_chars: HashSet<char> = a.chars().collect();
    let b_chars: HashSet<char> = b.chars().collect();

    let intersection = a_chars.intersection(&b_chars).count();
    let union = a_chars.union(&b_chars).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}