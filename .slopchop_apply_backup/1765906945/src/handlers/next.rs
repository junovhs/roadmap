//! Handler for the `next` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::types::TaskStatus;

pub fn handle(json: bool) -> Result<()> {
    let conn = Db::connect()?;
    let graph = TaskGraph::build(&conn)?;
    let critical_path = graph.get_critical_path();

    if json {
        return print_json(&critical_path);
    }

    print_human(&critical_path, &graph)
}

fn print_json(tasks: &[&roadmap::engine::types::Task]) -> Result<()> {
    let output: Vec<_> = tasks.iter().map(|t| {
        serde_json::json!({
            "id": t.id, "slug": t.slug, "title": t.title,
            "status": t.status.to_string(), "test_cmd": t.test_cmd
        })
    }).collect();
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_human(tasks: &[&roadmap::engine::types::Task], graph: &TaskGraph) -> Result<()> {
    println!("{} Next Actionable Tasks:", "??".cyan());

    if tasks.is_empty() {
        println!("   {} All tasks completed or none defined.", "(empty)".dimmed());
        return Ok(());
    }

    for task in tasks {
        let icon = status_icon(task.status);
        println!("   {} [{}] {}", icon, task.slug.yellow(), task.title);

        let blocked = graph.get_blocked_by(task.id);
        if !blocked.is_empty() {
            let names: Vec<_> = blocked.iter().map(|t| t.slug.as_str()).collect();
            println!("      {} {}", "�� blocks:".dimmed(), names.join(", ").dimmed());
        }
    }
    Ok(())
}

fn status_icon(status: TaskStatus) -> colored::ColoredString {
    match status {
        TaskStatus::Pending => "	".dimmed(),
        TaskStatus::Active => "?".yellow(),
        TaskStatus::Done => "�".green(),
        TaskStatus::Blocked => "?".red(),
    }
}