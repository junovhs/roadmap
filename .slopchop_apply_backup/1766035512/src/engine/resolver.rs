//! Fuzzy Task Resolver: Matches human queries to Task IDs.

use super::fuzzy::calculate_score;
use super::repo::{TaskRepo, TASK_SELECT};
use super::types::Task;
use anyhow::{bail, Result};
use rusqlite::{params, Connection, OptionalExtension};

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
            bail!("No exact match for '{}' in strict mode.", query);
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
            .ok_or_else(|| anyhow::anyhow!("No task matches '{}'", query))?;

        Ok(ResolveResult {
            task,
            confidence: 1.0,
        })
    }
}