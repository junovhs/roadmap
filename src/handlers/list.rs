//! Handler for the `list` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::runner::get_git_sha;
use roadmap::engine::types::DerivedStatus;

pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let tasks = repo.get_all()?;
    let head_sha = get_git_sha();

    println!("{} All Tasks:", "📋".cyan());

    if tasks.is_empty() {
        println!("   {} No tasks defined yet.", "(empty)".dimmed());
        return Ok(());
    }

    for task in tasks {
        let derived = task.derive_status(&head_sha);
        let icon = status_icon(derived);
        let test = if task.test_cmd.is_some() { " 🧪" } else { "" };
        println!(
            "   {} [{}] {} ({}){test}",
            icon,
            task.slug.blue(),
            task.title,
            derived.to_string().dimmed()
        );
    }

    Ok(())
}

fn status_icon(status: DerivedStatus) -> colored::ColoredString {
    match status {
        DerivedStatus::Unproven => "○".dimmed(),
        DerivedStatus::Proven => "✓".green(),
        DerivedStatus::Stale => "⟳".yellow(),
        DerivedStatus::Broken => "✗".red(),
        DerivedStatus::Attested => "◐".yellow(),
    }
}
