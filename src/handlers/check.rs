//! Handler for the `check` command.

use anyhow::{bail, Result};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::runner::{get_git_sha, VerifyRunner};
use roadmap::engine::types::{DerivedStatus, Proof, Task, TaskStatus};

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
        println!("{} No verification command defined.", "⚠".yellow());
        println!("   Use --force --reason \"...\" to mark as ATTESTED");
        return Ok(());
    };

    run_verification(&repo, &task, test_cmd, &head_sha)
}

fn handle_force(
    repo: &TaskRepo<&rusqlite::Connection>,
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
        "⚠".yellow(),
        task.slug.yellow()
    );
    println!("   {} \"{}\"", "reason:".dimmed(), reason);
    println!(
        "   {} sha={}",
        "proof:".dimmed(),
        &git_sha[..7.min(git_sha.len())]
    );

    show_unblocked(repo, task.id)
}

fn get_active_task(repo: &TaskRepo<&rusqlite::Connection>) -> Result<Task> {
    let Some(active_id) = repo.get_active_task_id()? else {
        bail!("No active task. Run `roadmap do <task>` first.");
    };

    let Some(task) = repo.find_by_id(active_id)? else {
        bail!("Active task not found in database.");
    };

    Ok(task)
}

fn run_verification(
    repo: &TaskRepo<&rusqlite::Connection>,
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
    repo: &TaskRepo<&rusqlite::Connection>,
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
    println!(
        "   {} sha={} duration={}ms",
        "proof:".dimmed(),
        &git_sha[..7.min(git_sha.len())],
        duration_ms
    );

    show_unblocked(repo, task.id)
}

#[allow(clippy::cast_possible_truncation)]
fn mark_broken(
    repo: &TaskRepo<&rusqlite::Connection>,
    task: &Task,
    cmd: &str,
    result: &roadmap::engine::runner::VerifyResult,
    git_sha: &str,
) -> Result<()> {
    let duration_ms = result.duration.as_millis() as u64;
    let exit_code = result.exit_code.unwrap_or(1);

    // Save the failed proof for history
    let proof = Proof::new(cmd, exit_code, git_sha, duration_ms);
    repo.save_proof(task.id, &proof)?;

    println!(
        "{} BROKEN! Task [{}] verification failed",
        "✗".red(),
        task.slug.red()
    );
    Ok(())
}

fn show_unblocked(repo: &TaskRepo<&rusqlite::Connection>, done_id: i64) -> Result<()> {
    let graph = TaskGraph::build(repo.conn())?;
    let available: Vec<_> = graph
        .get_frontier()
        .into_iter()
        .filter(|t| t.id != done_id)
        .take(3)
        .collect();

    if !available.is_empty() {
        println!("\n🎯 Now available:");
        for t in available {
            let status = t.derive_status(graph.head_sha());
            println!(
                "   ○ [{}] {} ({})",
                t.slug.yellow(),
                t.title,
                status.to_string().dimmed()
            );
        }
    }
    Ok(())
}
