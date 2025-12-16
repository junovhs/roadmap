//! Handler for the `add` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::{slugify, TaskResolver};

pub fn handle(title: &str, blocks: Option<&str>, after: Option<&str>, test_cmd: Option<&str>) -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);
    let slug = slugify(title);

    if repo.find_by_slug(&slug)?.is_some() {
        bail!("Task with slug '{slug}' already exists");
    }

    let task_id = match test_cmd {
        Some(cmd) => repo.add_with_test(&slug, title, cmd)?,
        None => repo.add(&slug, title)?,
    };

    println!("{} Added task [{}] {}", "�".green(), slug.yellow(), title);

    let resolver = TaskResolver::new(repo.conn());

    if let Some(after_ref) = after {
        link_after(&repo, &resolver, task_id, after_ref, &slug)?;
    }

    if let Some(blocks_ref) = blocks {
        link_blocks(&repo, &resolver, task_id, blocks_ref, &slug)?;
    }

    Ok(())
}

fn link_after(repo: &TaskRepo, resolver: &TaskResolver, task_id: i64, after_ref: &str, slug: &str) -> Result<()> {
    let after_task = resolver.resolve(after_ref)?;
    let graph = TaskGraph::build(repo.conn())?;

    if graph.would_create_cycle(after_task.task.id, task_id) {
        bail!("Adding this dependency would create a cycle!");
    }

    repo.link(after_task.task.id, task_id)?;
    println!("   {} [{}] blocks [{}]", "".cyan(), after_task.task.slug, slug);
    Ok(())
}

fn link_blocks(repo: &TaskRepo, resolver: &TaskResolver, task_id: i64, blocks_ref: &str, slug: &str) -> Result<()> {
    let blocks_task = resolver.resolve(blocks_ref)?;
    let graph = TaskGraph::build(repo.conn())?;

    if graph.would_create_cycle(task_id, blocks_task.task.id) {
        bail!("Adding this dependency would create a cycle!");
    }

    repo.link(task_id, blocks_task.task.id)?;
    println!("   {} [{}] blocks [{}]", "".cyan(), slug, blocks_task.task.slug);
    Ok(())
}