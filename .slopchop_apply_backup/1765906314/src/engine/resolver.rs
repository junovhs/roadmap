//! Fuzzy Task Resolver: Matches human-friendly names to task IDs.
//!
//! Resolution order:
//! 1. Exact ID match (numeric)
//! 2. Exact slug match (case-insensitive)
//! 3. Fuzzy match on title/slug (Levenshtein-like scoring)

use super::types::Task;
use anyhow::{bail, Result};
use rusqlite::Connection;

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
    /// Returns error if no match found or query is ambiguous.
    pub fn resolve(&self, query: &str) -> Result<ResolveResult> {
        let query = query.trim();

        // 1. Try exact ID match
        if let Ok(id) = query.parse::<i64>() {
            if let Some(task) = self.find_by_id(id)? {
                return Ok(ResolveResult {
                    task,
                    confidence: 1.0,
                    match_type: MatchType::ExactId,
                });
            }
        }

        // 2. Try exact slug match (case-insensitive)
        if let Some(task) = self.find_by_slug(query)? {
            return Ok(ResolveResult {
                task,
                confidence: 1.0,
                match_type: MatchType::ExactSlug,
            });
        }

        // 3. Fuzzy match
        let candidates = self.fuzzy_search(query)?;

        if candidates.is_empty() {
            bail!("No task found matching '{query}'");
        }

        // Check confidence threshold
        let best = &candidates[0];
        if best.confidence < 0.4 {
            let suggestions: Vec<String> = candidates
                .iter()
                .take(3)
                .map(|c| format!("  - [{}] {} ({:.0}%)", c.task.slug, c.task.title, c.confidence * 100.0))
                .collect();

            bail!(
                "Ambiguous query '{query}'. Did you mean:\n{}",
                suggestions.join("\n")
            );
        }

        // Warn if close alternatives exist
        if candidates.len() > 1 && candidates[1].confidence > 0.7 * best.confidence {
            eprintln!(
                "⚠ Multiple matches. Using '{}' ({}%). Alt: '{}' ({}%)",
                best.task.slug,
                (best.confidence * 100.0) as i32,
                candidates[1].task.slug,
                (candidates[1].confidence * 100.0) as i32,
            );
        }

        Ok(ResolveResult {
            task: best.task.clone(),
            confidence: best.confidence,
            match_type: MatchType::FuzzyMatch,
        })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Task>> {
        use super::types::TaskStatus;

        let result = self.conn.query_row(
            "SELECT id, slug, title, status, test_cmd, created_at FROM tasks WHERE id = ?1",
            [id],
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
        );

        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn find_by_slug(&self, slug: &str) -> Result<Option<Task>> {
        use super::types::TaskStatus;

        let result = self.conn.query_row(
            "SELECT id, slug, title, status, test_cmd, created_at FROM tasks WHERE LOWER(slug) = LOWER(?1)",
            [slug],
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
        );

        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn fuzzy_search(&self, query: &str) -> Result<Vec<ResolveResult>> {
        use super::types::TaskStatus;

        let mut stmt = self.conn.prepare(
            "SELECT id, slug, title, status, test_cmd, created_at FROM tasks"
        )?;

        let tasks: Vec<Task> = stmt
            .query_map([], |row| {
                let status_str: String = row.get(3)?;
                Ok(Task {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    title: row.get(2)?,
                    status: TaskStatus::from(status_str),
                    test_cmd: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();

        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<ResolveResult> = tasks
            .into_iter()
            .map(|task| {
                let score = calculate_match_score(&task, &query_lower, &query_words);
                ResolveResult {
                    task,
                    confidence: score,
                    match_type: MatchType::FuzzyMatch,
                }
            })
            .filter(|r| r.confidence > 0.0)
            .collect();

        results.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }
}

/// Calculates a match score between a task and a query.
fn calculate_match_score(task: &Task, query: &str, query_words: &[&str]) -> f64 {
    let slug_lower = task.slug.to_lowercase();
    let title_lower = task.title.to_lowercase();

    let mut score = 0.0;

    // Substring match (high value)
    if slug_lower.contains(query) {
        score += 0.8;
    }
    if title_lower.contains(query) {
        score += 0.7;
    }

    // Word-level matching
    for word in query_words {
        if slug_lower.contains(word) {
            score += 0.3;
        }
        if title_lower.contains(word) {
            score += 0.25;
        }
    }

    // Prefix match bonus
    if slug_lower.starts_with(query) {
        score += 0.5;
    }

    // Levenshtein-like similarity for slug
    let slug_sim = string_similarity(&slug_lower, query);
    score += slug_sim * 0.4;

    // Normalize (cap at 1.0)
    score.min(1.0)
}

/// Simple string similarity (Jaccard-like on characters).
fn string_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let a_chars: std::collections::HashSet<char> = a.chars().collect();
    let b_chars: std::collections::HashSet<char> = b.chars().collect();

    let intersection = a_chars.intersection(&b_chars).count();
    let union = a_chars.union(&b_chars).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Add Dark Mode"), "add-dark-mode");
        assert_eq!(slugify("Fix Bug #123"), "fix-bug-123");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_string_similarity() {
        assert!(string_similarity("auth", "authentication") > 0.5);
        assert!(string_similarity("xyz", "abc") < 0.2);
    }
}
