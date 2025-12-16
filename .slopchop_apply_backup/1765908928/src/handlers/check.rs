//! Handler for the `check` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::runner::VerifyRunner;
use roadmap::engine::types::TaskStatus;

pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);

    let task = get_active_task(&repo)?;
    println!("?? Checking: [{}] {}", task.slug.yellow(), task.title);

    let Some(test_cmd) = &task.test_cmd else {
        println!("{} No verification command defined.", "?".yellow());
        println!("   Run with --force to mark complete, or add a test:");
        println!("   roadmap edit {} --test \"your_test_cmd\"", task.slug);
        return Ok(());
    };

    run_verification(&repo, &task, test_cmd)
}

fn get_active_task(repo: &TaskRepo) -> Result<roadmap::engine::types::Task> {
    let Some(active_id) = repo.get_active_task_id()? else {
        bail!("No active task. Run `roadmap do <task>` first.");
    };

    let Some(task) = repo.find_by_id(active_id)? else {
        bail!("Active task not found in database.");
    };

    Ok(task)
}

fn run_verification(repo: &TaskRepo, task: &roadmap::engine::types::Task, test_cmd: &str) -> Result<()> {
    println!("   {} {}", "running:".dimmed(), test_cmd);

    let runner = VerifyRunner::default_runner();
    let result = runner.verify(test_cmd)?;

    if result.passed() {
        mark_done(repo, task)
    } else {
        println!("{} Verification failed. Task remains {}.", "?".red(), "ACTIVE".yellow());
        Ok(())
    }
}

fn mark_done(repo: &TaskRepo, task: &roadmap::engine::types::Task) -> Result<()> {
    repo.update_status(task.id, TaskStatus::Done)?;
    println!("{} Verified! Task [{}] marked DONE", "�".green(), task.slug.green());

    let graph = TaskGraph::build(repo.conn())?;
    let available: Vec<_> = graph.get_critical_path().into_iter()
        .filter(|t| t.id != task.id)
        .take(3)
        .collect();

    if !available.is_empty() {
        println!("\n?? Now available:");
        for t in available {
            println!("   	 [{}] {}", t.slug.yellow(), t.title);
        }
    }
    Ok(())
}