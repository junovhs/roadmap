//! Handler for the `add` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::{slugify, TaskResolver};

/// Handles adding a new task and its dependencies.
///
/// # Errors
/// Returns error if task exists, database is locked, or dependency creates a cycle.
pub fn handle(
    title: &str,
    blocks: Option<&str>,
    after: Option<&str>,
    test_cmd: Option<&str>,
    scopes: Option<Vec<String>>,
) -> Result<()> {
    let mut conn = Db::connect()?;
    let slug = slugify(title);

    let tx = conn.transaction()?;
    let repo = TaskRepo::new(&tx);

    if repo.find_by_slug(&slug)?.is_some() {
        bail!("Task with slug '{slug}' already exists");
    }

    let task_id = repo.add(&slug, title, test_cmd)?;

    if let Some(scope_list) = scopes {
        for scope in scope_list {
            repo.add_scope(task_id, &scope)?;
        }
    }

    if let Some(after_ref) = after {
        let resolver = TaskResolver::new(&tx);
        let after_task = resolver.resolve(after_ref)?;

        let graph = TaskGraph::build(&tx)?;
        if graph.would_create_cycle(after_task.task.id, task_id) {
            bail!("Adding this dependency would create a cycle!");
        }

        repo.link(after_task.task.id, task_id)?;
        println!(
            "   {} [{}] blocks [{}]",
            " ".cyan(),
            after_task.task.slug,
            slug
        );
    }

    if let Some(blocks_ref) = blocks {
        let resolver = TaskResolver::new(&tx);
        let blocks_task = resolver.resolve(blocks_ref)?;

        let graph = TaskGraph::build(&tx)?;
        if graph.would_create_cycle(task_id, blocks_task.task.id) {
            bail!("Adding this dependency would create a cycle!");
        }

        repo.link(task_id, blocks_task.task.id)?;
        println!(
            "   {} [{}] blocks [{}]",
            " ".cyan(),
            slug,
            blocks_task.task.slug
        );
    }

    tx.commit()?;
    println!("{} Added task [{}] {}", "✓".green(), slug.yellow(), title);
    Ok(())
}