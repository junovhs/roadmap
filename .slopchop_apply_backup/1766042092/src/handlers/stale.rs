//! Handler for the `stale` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::runner::get_git_sha;
use roadmap::engine::types::DerivedStatus;

/// Scans for and lists all tasks with stale proofs.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let tasks = repo.get_all()?;
    let head_sha = get_git_sha();
    let short_head = &head_sha[..7.min(head_sha.len())];

    let stale_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| matches!(t.derive_status(&head_sha), DerivedStatus::Stale))
        .collect();

    if stale_tasks.is_empty() {
        println!("{} No stale tasks found. The truth is fresh.", "✓".green());
        return Ok(());
    }

    println!("{} Found {} stale tasks:", "⚡".yellow(), stale_tasks.len());
    println!("   Current HEAD: {}", short_head.dimmed());
    println!();

    for task in stale_tasks {
        let proof = task.proof.as_ref().expect("Stale task must have proof");
        let proof_sha = &proof.git_sha[..7.min(proof.git_sha.len())];

        println!(
            "   [{}] {}",
            task.slug.yellow().bold(),
            task.title
        );
        println!(
            "     last proven at: {}  (diff: {})",
            proof_sha.dimmed(),
            "HEAD moved".red()
        );
    }

    Ok(())
}