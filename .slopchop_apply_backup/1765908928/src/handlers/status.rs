//! Handler for the `status` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::types::TaskStatus;

pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);
    let graph = TaskGraph::build(repo.conn())?;

    let all = repo.get_all()?;
    let done = all.iter().filter(|t| t.status == TaskStatus::Done).count();

    println!("{} Roadmap Status", "??".cyan());
    println!("   Tasks: {}/{} complete", done, all.len());
    println!("   Graph: {} nodes, {} edges", graph.task_count(), graph.edge_count());

    print_focus(&repo)?;
    print_next(&graph);

    Ok(())
}

fn print_focus(repo: &TaskRepo) -> Result<()> {
    if let Some(id) = repo.get_active_task_id()? {
        if let Some(task) = repo.find_by_id(id)? {
            println!("\n{} Focus: [{}] {}", "?".yellow(), task.slug.yellow(), task.title);
            return Ok(());
        }
    }
    println!("\n{} No active task", "	".dimmed());
    Ok(())
}

fn print_next(graph: &TaskGraph) {
    let critical = graph.get_critical_path();
    if !critical.is_empty() {
        println!("\n{} Next up:", "".cyan());
        for task in critical.iter().take(3) {
            println!("   	 [{}] {}", task.slug.dimmed(), task.title);
        }
    }
}