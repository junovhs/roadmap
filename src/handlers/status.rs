//! Handler for the `status` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::context::RepoContext;
use roadmap::engine::db::Db;
use roadmap::engine::graph::{StatusCounts, TaskGraph};
use roadmap::engine::repo::TaskRepo;
use serde::Serialize;

/// Displays the current project status.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle(json: bool) -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let graph = TaskGraph::build(&conn)?;
    let context = RepoContext::new()?;
    
    if json {
        return print_json(&repo, &graph, &context);
    }

    print_human(&repo, &graph, &context)
}

#[derive(Serialize)]
struct StatusReport {
    head_sha: String,
    counts: StatusCounts,
    focus: Option<TaskView>,
    frontier: Vec<TaskView>,
}

#[derive(Serialize)]
struct TaskView {
    id: i64,
    slug: String,
    title: String,
    status: String,
}

fn print_json(repo: &TaskRepo<'_>, graph: &TaskGraph, context: &RepoContext) -> Result<()> {
    let head_sha = context.head_sha().to_string();
    let counts = graph.status_counts();
    
    let focus = if let Some(id) = repo.get_active_task_id()? {
        repo.find_by_id(id)?.map(|t| {
            let status = t.derive_status(context);
            TaskView {
                id: t.id,
                slug: t.slug,
                title: t.title,
                status: format!("{status:?}"),
            }
        })
    } else {
        None
    };

    let frontier = graph.get_frontier().into_iter().take(5).map(|t| {
        let status = t.derive_status(context);
        TaskView {
            id: t.id,
            slug: t.slug.clone(),
            title: t.title.clone(),
            status: format!("{status:?}"),
        }
    }).collect();

    let report = StatusReport {
        head_sha,
        counts,
        focus,
        frontier,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn print_human(repo: &TaskRepo<'_>, graph: &TaskGraph, context: &RepoContext) -> Result<()> {
    let head_sha = context.head_sha();

    println!("{} Roadmap Status", "📊".cyan());

    if let Some(id) = repo.get_active_task_id()? {
        if let Some(task) = repo.find_by_id(id)? {
            println!(
                "   Focus: [{}] {} ({})",
                task.slug.yellow(),
                task.title,
                task.derive_status(context).to_string().dimmed()
            );
        }
    }

    let frontier = graph.get_frontier();
    if !frontier.is_empty() {
        println!("\n   Next up:");
        for task in frontier.iter().take(3) {
            println!("     - [{}] {}", task.slug.dimmed(), task.title);
        }
    }

    println!("\n   Repo HEAD: {}", &head_sha[..7.min(head_sha.len())].dimmed());

    Ok(())
}