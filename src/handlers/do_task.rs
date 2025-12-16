//! Handler for the `do` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::TaskResolver;
use roadmap::engine::runner::get_git_sha;
use roadmap::engine::types::{DerivedStatus, TaskStatus};

pub fn handle(task_ref: &str, strict: bool) -> Result<()> {
    let conn = Db::connect()?;
    let head_sha = get_git_sha();

    let resolver = if strict {
        TaskResolver::strict(&conn)
    } else {
        TaskResolver::new(&conn)
    };

    let result = resolver.resolve(task_ref)?;
    let task = &result.task;

    check_not_blocked(&conn, task, &head_sha)?;

    let repo = TaskRepo::new(&conn);
    repo.update_status(task.id, TaskStatus::Active)?;
    repo.set_active_task(task.id)?;

    let derived = task.derive_status(&head_sha);
    println!(
        "{} Now working on: [{}] {} ({})",
        "→".yellow(),
        task.slug.yellow(),
        task.title,
        derived.to_string().dimmed()
    );

    if let Some(ref cmd) = task.test_cmd {
        println!("   {} {}", "verify:".dimmed(), cmd.dimmed());
    }

    Ok(())
}

fn check_not_blocked(conn: &rusqlite::Connection, task: &roadmap::engine::types::Task, head_sha: &str) -> Result<()> {
    let graph = TaskGraph::build(conn)?;
    let blockers = graph.get_blockers(task.id);

    let incomplete: Vec<_> = blockers
        .iter()
        .filter(|t| {
            let status = t.derive_status(head_sha);
            !matches!(status, DerivedStatus::Proven | DerivedStatus::Attested)
        })
        .collect();

    if !incomplete.is_empty() {
        let names: Vec<_> = incomplete
            .iter()
            .map(|t| format!("{} ({})", t.slug, t.derive_status(head_sha)))
            .collect();
        bail!(
            "Task [{}] is blocked by: {}",
            task.slug,
            names.join(", ")
        );
    }

    Ok(())
}
