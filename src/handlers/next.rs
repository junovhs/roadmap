//! Handler for the `next` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::types::DerivedStatus;

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

fn print_json(tasks: &[&roadmap::engine::types::Task], head_sha: &str) -> Result<()> {
    let output: Vec<_> = tasks
        .iter()
        .map(|t| {
            let derived = t.derive_status(head_sha);
            serde_json::json!({
                "id": t.id,
                "slug": t.slug,
                "title": t.title,
                "status": derived.to_string(),
                "test_cmd": t.test_cmd
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_human(tasks: &[&roadmap::engine::types::Task], graph: &TaskGraph) {
    println!("{} Actionable Tasks (frontier):", "??".cyan());

    if tasks.is_empty() {
        println!(
            "   {} All claims proven or none defined.",
            "(empty)".dimmed()
        );
        return;
    }

    for task in tasks {
        let derived = task.derive_status(graph.head_sha());
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
                "      {} {}",
                "? unblocks:".dimmed(),
                names.join(", ").dimmed()
            );
        }
    }
}

fn status_icon(status: DerivedStatus) -> colored::ColoredString {
    match status {
        DerivedStatus::Broken => "?".red(),
        DerivedStatus::Stale => "?".yellow(),
        DerivedStatus::Unproven => "	".dimmed(),
        DerivedStatus::Proven => "�".green(),
        DerivedStatus::Attested => "?".blue(),
    }
}