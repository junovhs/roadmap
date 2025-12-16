//! Fuzzy string matching utilities for task resolution.

use super::types::Task;
use std::collections::HashSet;

/// Calculates a match score between a task and a query.
pub fn calculate_score(task: &Task, query: &str, query_words: &[&str]) -> f64 {
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

    // Similarity score for slug
    let slug_sim = string_similarity(&slug_lower, query);
    score += slug_sim * 0.4;

    score.min(1.0)
}

/// Simple string similarity (Jaccard-like on characters).
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
    }

    #[test]
    fn test_string_similarity() {
        assert!(string_similarity("auth", "authentication") > 0.5);
        assert!(string_similarity("xyz", "abc") < 0.2);
    }
}