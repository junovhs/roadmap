//! Handler for the `stale` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::context::RepoContext;
use roadmap::engine::db::Db;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::types::DerivedStatus;
use serde::Serialize;

/// Scans for and lists all tasks with stale proofs.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle(json: bool) -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let tasks = repo.get_all()?;
    let context = RepoContext::new()?;
    let head_sha = context.head_sha();

    let stale_tasks: Vec<_> = tasks
        .into_iter()
        .filter(|t| matches!(t.derive_status(&context), DerivedStatus::Stale))
        .collect();

    if json {
        return print_json(&stale_tasks, head_sha);
    }

    print_human(&stale_tasks, head_sha);
    Ok(())
}

#[derive(Serialize)]
struct StaleReport {
    head_sha: String,
    stale_count: usize,
    tasks: Vec<StaleTaskView>,
}

#[derive(Serialize)]
struct StaleTaskView {
    id: i64,
    slug: String,
    title: String,
    proof_sha: Option<String>,
}

fn print_json(tasks: &[roadmap::engine::types::Task], head_sha: &str) -> Result<()> {
    let views: Vec<StaleTaskView> = tasks.iter().map(|t| {
        StaleTaskView {
            id: t.id,
            slug: t.slug.clone(),
            title: t.title.clone(),
            proof_sha: t.proof.as_ref().map(|p| p.git_sha.clone()),
        }
    }).collect();

    let report = StaleReport {
        head_sha: head_sha.to_string(),
        stale_count: tasks.len(),
        tasks: views,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn print_human(tasks: &[roadmap::engine::types::Task], head_sha: &str) {
    let short_head = &head_sha[..7.min(head_sha.len())];

    if tasks.is_empty() {
        println!("{} No stale tasks found. The truth is fresh.", "✓".green());
        return;
    }

    println!("{} Found {} stale tasks:", "⚡".yellow(), tasks.len());
    println!("   Current HEAD: {}", short_head.dimmed());
    println!();

    for task in tasks {
        if let Some(proof) = &task.proof {
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
    }
}