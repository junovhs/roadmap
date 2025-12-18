//! Handler for the `check` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::runner::{get_git_sha, VerifyRunner};
use roadmap::engine::types::{Proof, Task, TaskStatus};

/// Runs verification for the active task.
///
/// # Errors
/// Returns error if no task is active or database fails.
pub fn handle(force: bool, reason: Option<&str>) -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let head_sha = get_git_sha();

    let task = get_active_task(&repo)?;
    let derived = task.derive_status(&head_sha);

    println!(
        "🔍 Checking: [{}] {} ({})",
        task.slug.yellow(),
        task.title,
        derived.to_string().dimmed()
    );

    if force {
        return handle_force(&repo, &task, reason, &head_sha);
    }

    let Some(test_cmd) = &task.test_cmd else {
        println!("{} No verification command defined.", "?".yellow());
        println!("   Use --force --reason \"...\" to mark as ATTESTED");
        return Ok(());
    };

    run_verification(&repo, &task, test_cmd, &head_sha)
}

fn handle_force(
    repo: &TaskRepo<'_>,
    task: &Task,
    reason: Option<&str>,
    git_sha: &str,
) -> Result<()> {
    let reason = reason.unwrap_or("Manual attestation");
    let proof = Proof::attested(reason, git_sha);
    repo.save_proof(task.id, &proof)?;
    repo.update_status(task.id, TaskStatus::Attested)?;

    println!(
        "{} Task [{}] marked ATTESTED (not verified)",
        "!".yellow(),
        task.slug.yellow()
    );
    show_unblocked(repo, task.id)
}

fn get_active_task(repo: &TaskRepo<'_>) -> Result<Task> {
    let Some(active_id) = repo.get_active_task_id()? else {
        bail!("No active task. Run `roadmap do <task>` first.");
    };
    repo.find_by_id(active_id)?
        .ok_or_else(|| anyhow::anyhow!("Active task not found"))
}

fn run_verification(
    repo: &TaskRepo<'_>,
    task: &Task,
    test_cmd: &str,
    head_sha: &str,
) -> Result<()> {
    println!("   {} {}", "running:".dimmed(), test_cmd);
    let runner = VerifyRunner::default_runner();
    let result = runner.verify(test_cmd)?;

    if result.passed() {
        mark_proven(repo, task, test_cmd, &result, head_sha)
    } else {
        mark_broken(repo, task, test_cmd, &result, head_sha)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn mark_proven(
    repo: &TaskRepo<'_>,
    task: &Task,
    cmd: &str,
    result: &roadmap::engine::runner::VerifyResult,
    git_sha: &str,
) -> Result<()> {
    let duration_ms = result.duration.as_millis() as u64;
    let exit_code = result.exit_code.unwrap_or(0);

    let proof = Proof::new(cmd, exit_code, git_sha, duration_ms);
    repo.save_proof(task.id, &proof)?;
    repo.update_status(task.id, TaskStatus::Done)?;

    println!(
        "{} PROVEN! Task [{}] verified",
        "✓".green(),
        task.slug.green()
    );
    show_unblocked(repo, task.id)
}

#[allow(clippy::cast_possible_truncation)]
fn mark_broken(
    repo: &TaskRepo<'_>,
    task: &Task,
    cmd: &str,
    result: &roadmap::engine::runner::VerifyResult,
    git_sha: &str,
) -> Result<()> {
    let duration_ms = result.duration.as_millis() as u64;
    let exit_code = result.exit_code.unwrap_or(1);

    let proof = Proof::new(cmd, exit_code, git_sha, duration_ms);
    repo.save_proof(task.id, &proof)?;

    println!(
        "{} BROKEN! Task [{}] verification failed",
        "✗".red(),
        task.slug.red()
    );
    Ok(())
}

fn show_unblocked(repo: &TaskRepo<'_>, done_id: i64) -> Result<()> {
    // Note: TaskGraph::build takes &Connection, repo.conn() not exposed, 
    // so we re-open or just rely on status/next for UI.
    // For now, simpler output to avoid complexity violations.
    println!("   {} Check `roadmap next` for available tasks", "ℹ".blue());
    Ok(())
}