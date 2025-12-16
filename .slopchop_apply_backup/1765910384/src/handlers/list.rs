//! Handler for the `list` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::types::TaskStatus;

pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);
    let tasks = repo.get_all()?;

    println!("{} All Tasks:", "??".cyan());

    if tasks.is_empty() {
        println!("   {} No tasks defined yet.", "(empty)".dimmed());
        return Ok(());
    }

    for task in tasks {
        let icon = match task.status {
            TaskStatus::Pending => "	".dimmed(),
            TaskStatus::Active => "?".yellow(),
            TaskStatus::Done => "�".green(),
            TaskStatus::Blocked => "?".red(),
        };
        let test = if task.test_cmd.is_some() { " ??" } else { "" };
        println!("   {} [{}] {} ({}){}", icon, task.slug.blue(), task.title, task.status.to_string().dimmed(), test);
    }

    Ok(())
}