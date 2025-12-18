//! Handler for the `list` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::runner::get_git_sha;

/// Lists all tasks in the repository.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let tasks = repo.get_all()?;
    let head_sha = get_git_sha();

    println!("{} All Tasks:", "📋".cyan());

    for task in tasks {
        let derived = task.derive_status(&head_sha);
        println!(
            "   [{}] {} ({})",
            task.slug.blue(),
            task.title,
            derived.to_string().dimmed()
        );
    }
    Ok(())
}