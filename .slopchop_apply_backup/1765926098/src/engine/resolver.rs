//! Fuzzy Task Resolver: Matches human-friendly names to task IDs.

use super::fuzzy::calculate_score;
use super::repo::{row_to_task, TaskRepo};
use super::types::Task;
use anyhow::{bail, Result};
use rusqlite::Connection;
use std::ops::Deref;

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
pub struct TaskResolver<'a, C: Deref<Target = Connection>> {
    repo: TaskRepo<&'a C>,
    strict: bool,
}

impl<'a, C: Deref<Target = Connection>> TaskResolver<'a, C> {
    #[must_use]
    pub fn new(conn: &'a C) -> Self {
        Self {
            repo: TaskRepo::new(conn),
            strict: false,
        }
    }

    /// Creates a resolver in strict mode (for agents/JSON output).
    /// In strict mode, fuzzy matching is disabled - only exact matches work.
    #[must_use]
    pub fn strict(conn: &'a C) -> Self {
        Self {
            repo: TaskRepo::new(conn),
            strict: true,
        }
    }

    /// Resolves a query to a task with confidence scoring.
    ///
    /// # Errors
    /// Returns error if no task matches or query is ambiguous.
    pub fn resolve(&self, query: &str) -> Result<ResolveResult> {
        let query = query.trim();

        // Try exact ID match
        if let Ok(id) = query.parse::<i64>() {
            if let Some(task) = self.repo.find_by_id(id)? {
                return Ok(ResolveResult {
                    task,
                    confidence: 1.0,
                    match_type: MatchType::ExactId,
                });
            }
        }

        // Try exact slug match (case-insensitive)
        if let Some(task) = self.repo.find_by_slug_ci(query)? {
            return Ok(ResolveResult {
                task,
                confidence: 1.0,
                match_type: MatchType::ExactSlug,
            });
        }

        if self.strict {
            self.strict_error(query)
        } else {
            self.fuzzy_resolve(query)
        }
    }

    fn strict_error(&self, query: &str) -> Result<ResolveResult> {
        let candidates = self.fuzzy_search(query)?;
        let suggestions = format_suggestions_json(&candidates);
        bail!("No exact match for '{query}'. Use ID or exact slug.\nCandidates:\n{suggestions}");
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

    fn fuzzy_search(&self, query: &str) -> Result<Vec<ResolveResult>> {
        let tasks = self.repo.get_all()?;
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<ResolveResult> = tasks
            .into_iter()
            .map(|task| {
                let score = calculate_score(&task, &query_lower, &query_words);
                ResolveResult {
                    task,
                    confidence: score,
                    match_type: MatchType::FuzzyMatch,
                }
            })
            .filter(|r| r.confidence > 0.0)
            .collect();

        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }
}

fn format_suggestions(candidates: &[ResolveResult]) -> String {
    candidates
        .iter()
        .take(3)
        .map(|c| format!("  - [{}] {}", c.task.slug, c.task.title))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_suggestions_json(candidates: &[ResolveResult]) -> String {
    candidates
        .iter()
        .take(5)
        .map(|c| format!("  {{\"id\": {}, \"slug\": \"{}\"}}", c.task.id, c.task.slug))
        .collect::<Vec<_>>()
        .join("\n")
}
