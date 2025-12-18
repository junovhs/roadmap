//! Handler for the `list` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::context::RepoContext;
use roadmap::engine::db::Db;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::types::Task;
use serde::Serialize;

/// Lists all tasks in the repository.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle(json: bool) -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let tasks = repo.get_all()?;
    let context = RepoContext::new()?;

    if json {
        return print_json(&tasks, &context);
    }

    println!("{} All Tasks:", "📋".cyan());

    for task in tasks {
        let derived = task.derive_status(&context);
        println!(
            "   [{}] {} ({})",
            task.slug.blue(),
            task.title,
            derived.to_string().dimmed()
        );
    }
    Ok(())
}

#[derive(Serialize)]
struct TaskView {
    id: i64,
    slug: String,
    title: String,
    status: String,
    test_cmd: Option<String>,
    scopes: Vec<String>,
}

fn print_json(tasks: &[Task], context: &RepoContext) -> Result<()> {
    let views: Vec<TaskView> = tasks.iter().map(|t| {
        let status = t.derive_status(context);
        TaskView {
            id: t.id,
            slug: t.slug.clone(),
            title: t.title.clone(),
            status: format!("{status:?}"), // Serialize enum variant name
            test_cmd: t.test_cmd.clone(),
            scopes: t.scopes.clone(),
        }
    }).collect();

    println!("{}", serde_json::to_string_pretty(&views)?);
    Ok(())
}