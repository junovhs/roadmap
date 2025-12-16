//! Handler for the `do` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::TaskResolver;
use roadmap::engine::types::TaskStatus;

pub fn handle(task_ref: &str) -> Result<()> {
    let conn = Db::connect()?;
    let resolver = TaskResolver::new(&conn);
    let result = resolver.resolve(task_ref)?;
    let task = &result.task;

    check_not_blocked(&conn, task)?;

    let repo = TaskRepo::new(conn);
    repo.update_status(task.id, TaskStatus::Active)?;
    repo.set_active_task(task.id)?;

    println!("{} Now working on: [{}] {}", "?".yellow(), task.slug.yellow(), task.title);

    if let Some(ref cmd) = task.test_cmd {
        println!("   {} {}", "verify:".dimmed(), cmd.dimmed());
    }

    Ok(())
}

fn check_not_blocked(conn: &rusqlite::Connection, task: &roadmap::engine::types::Task) -> Result<()> {
    let graph = TaskGraph::build(conn)?;
    let blockers = graph.get_blockers(task.id);
    let active: Vec<_> = blockers.iter()
        .filter(|t| t.status != TaskStatus::Done)
        .collect();

    if !active.is_empty() {
        let names: Vec<_> = active.iter().map(|t| t.slug.as_str()).collect();
        bail!("Task [{}] is blocked by: {}", task.slug, names.join(", "));
    }

    Ok(())
}