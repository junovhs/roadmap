//! Handler for the `add` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::{slugify, TaskResolver};

pub fn handle(
    title: &str,
    blocks: Option<&str>,
    after: Option<&str>,
    test_cmd: Option<&str>,
) -> Result<()> {
    let mut conn = Db::connect()?;
    let slug = slugify(title);

    // Start transaction for atomic add + deps + cycle check
    let tx = conn.transaction()?;
    let repo = TaskRepo::new(&tx);

    // Check for duplicate
    if repo.find_by_slug(&slug)?.is_some() {
        bail!("Task with slug '{slug}' already exists");
    }

    // Insert task
    let task_id = repo.add(&slug, title, test_cmd)?;

    // Handle --after dependency
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
            "→".cyan(),
            after_task.task.slug,
            slug
        );
    }

    // Handle --blocks dependency
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
            "→".cyan(),
            slug,
            blocks_task.task.slug
        );
    }

    // Commit only if everything succeeded
    tx.commit()?;

    println!("{} Added task [{}] {}", "✓".green(), slug.yellow(), title);
    Ok(())
}
