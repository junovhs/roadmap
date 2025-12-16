//! Handler for the `status` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::types::DerivedStatus;

pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let graph = TaskGraph::build(repo.conn())?;

    let counts = graph.status_counts();
    let head_sha = graph.head_sha();

    println!("{} Roadmap Status", "📊".cyan());
    println!(
        "   {} {}/{}",
        "proven:".green(),
        counts.proven,
        counts.total()
    );
    if counts.attested > 0 {
        println!("   {} {}", "attested:".yellow(), counts.attested);
    }
    if counts.stale > 0 {
        println!("   {} {}", "stale:".yellow(), counts.stale);
    }
    if counts.broken > 0 {
        println!("   {} {}", "broken:".red(), counts.broken);
    }
    println!("   {} {}", "unproven:".dimmed(), counts.unproven);
    println!(
        "   {} {} nodes, {} edges",
        "graph:".dimmed(),
        graph.task_count(),
        graph.edge_count()
    );
    println!(
        "   {} {}",
        "HEAD:".dimmed(),
        &head_sha[..7.min(head_sha.len())]
    );

    print_focus(&repo, &head_sha)?;
    print_next(&graph);

    Ok(())
}

fn print_focus(repo: &TaskRepo<&rusqlite::Connection>, head_sha: &str) -> Result<()> {
    if let Some(id) = repo.get_active_task_id()? {
        if let Some(task) = repo.find_by_id(id)? {
            let derived = task.derive_status(head_sha);
            println!(
                "\n{} Focus: [{}] {} ({})",
                "→".yellow(),
                task.slug.yellow(),
                task.title,
                derived.to_string().dimmed()
            );
            return Ok(());
        }
    }
    println!("\n{} No active task", "○".dimmed());
    Ok(())
}

fn print_next(graph: &TaskGraph) {
    let frontier = graph.get_frontier();
    if !frontier.is_empty() {
        println!("\n{} Next up:", "🎯".cyan());
        for task in frontier.iter().take(3) {
            let derived = task.derive_status(graph.head_sha());
            println!(
                "   {} [{}] {} ({})",
                status_icon(derived),
                task.slug.dimmed(),
                task.title,
                derived.to_string().dimmed()
            );
        }
    }
}

fn status_icon(status: DerivedStatus) -> colored::ColoredString {
    match status {
        DerivedStatus::Broken => "✗".red(),
        DerivedStatus::Stale => "⟳".yellow(),
        DerivedStatus::Unproven => "○".dimmed(),
        DerivedStatus::Proven => "✓".green(),
        DerivedStatus::Attested => "◐".yellow(),
    }
}
