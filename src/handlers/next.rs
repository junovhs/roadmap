//! Handler for the `next` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::context::RepoContext;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::types::{DerivedStatus, Task};

/// Shows the frontier of actionable tasks.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle(json: bool) -> Result<()> {
    let conn = Db::connect()?;
    let graph = TaskGraph::build(&conn)?;
    let frontier = graph.get_frontier();

    if json {
        return print_json(&frontier, graph.head_sha());
    }

    print_human(&frontier, &graph);
    Ok(())
}

fn print_json(tasks: &[&Task], head_sha: &str) -> Result<()> {
    // Reconstruct context from the provided SHA to derive status for JSON output.
    // This allows agents to see if a task is Unproven vs Stale.
    let context = RepoContext::from_sha(head_sha.to_string());

    let output: Vec<_> = tasks
        .iter()
        .map(|t| {
            let status = t.derive_status(&context);
            serde_json::json!({
                "id": t.id,
                "slug": t.slug,
                "title": t.title,
                "status": status.to_string(),
                "test_cmd": t.test_cmd
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_human(tasks: &[&Task], graph: &TaskGraph) {
    println!("{} Actionable Tasks (frontier):", "🚀".cyan());

    if tasks.is_empty() {
        println!("   (All claims proven or none defined)");
        return;
    }

    // We can assume graph.head_sha() is consistent with the context used to build the graph.
    // Ideally TaskGraph would expose its context, but constructing one here is low cost.
    let context = RepoContext::from_sha(graph.head_sha().to_string());

    for task in tasks {
        let derived = task.derive_status(&context);
        let icon = status_icon(derived);
        println!(
            "   {} [{}] {} ({})",
            icon,
            task.slug.yellow(),
            task.title,
            derived.to_string().dimmed()
        );

        let blocked = graph.get_blocked_by(task.id);
        if !blocked.is_empty() {
            let names: Vec<_> = blocked.iter().map(|t| t.slug.as_str()).collect();
            println!(
                "      ℹ unblocks: {}",
                names.join(", ").dimmed()
            );
        }
    }
}

fn status_icon(status: DerivedStatus) -> colored::ColoredString {
    match status {
        DerivedStatus::Broken => "✗".red(),
        DerivedStatus::Stale => "⚡".yellow(),
        DerivedStatus::Unproven => "○".dimmed(),
        DerivedStatus::Proven => "✓".green(),
        DerivedStatus::Attested => "!".blue(),
    }
}