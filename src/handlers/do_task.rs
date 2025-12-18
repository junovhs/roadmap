//! Handler for the `do` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::context::RepoContext;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::TaskResolver;
use roadmap::engine::types::{DerivedStatus, TaskStatus};

/// Sets a task as the active focus.
///
/// # Errors
/// Returns error if task is blocked or not found.
pub fn handle(task_ref: &str, strict: bool) -> Result<()> {
    let conn = Db::connect()?;
    let context = RepoContext::new()?;

    let resolver = if strict {
        TaskResolver::strict(&conn)
    } else {
        TaskResolver::new(&conn)
    };

    let result = resolver.resolve(task_ref)?;
    let task = &result.task;

    check_not_blocked(&conn, task, &context)?;

    let repo = TaskRepo::new(&conn);
    repo.update_status(task.id, TaskStatus::Active)?;
    repo.set_active_task(task.id)?;

    println!(
        "{} Now working on: [{}] {}",
        "→".yellow(),
        task.slug.yellow(),
        task.title
    );

    Ok(())
}

fn check_not_blocked(
    conn: &rusqlite::Connection,
    task: &roadmap::engine::types::Task,
    context: &RepoContext,
) -> Result<()> {
    let graph = TaskGraph::build(conn)?;
    let blockers = graph.get_blockers(task.id);

    let incomplete: Vec<_> = blockers
        .into_iter()
        .filter(|t| {
            let status = t.derive_status(context);
            !matches!(status, DerivedStatus::Proven | DerivedStatus::Attested)
        })
        .collect();

    if !incomplete.is_empty() {
        let names: Vec<_> = incomplete.iter().map(|t| t.slug.as_str()).collect();
        bail!("Task [{}] is blocked by: {}", task.slug, names.join(", "));
    }
    Ok(())
}