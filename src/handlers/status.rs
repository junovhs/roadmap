//! Handler for the `status` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;

/// Displays the current project status.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let graph = TaskGraph::build(&conn)?;
    let head_sha = graph.head_sha();

    println!("{} Roadmap Status", "📊".cyan());

    if let Some(id) = repo.get_active_task_id()? {
        if let Some(task) = repo.find_by_id(id)? {
            println!(
                "   Focus: [{}] {} ({})",
                task.slug.yellow(),
                task.title,
                task.derive_status(head_sha).to_string().dimmed()
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

    Ok(())
}