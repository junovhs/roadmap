//! Handler for the `why` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::TaskResolver;
use roadmap::engine::runner::get_git_sha;
use roadmap::engine::types::{DerivedStatus, Proof};

/// Explains the status of a task and shows its audit log.
///
/// # Errors
/// Returns error if task resolution or DB query fails.
pub fn handle(task_ref: &str) -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(&conn);
    let head_sha = get_git_sha();

    let resolver = TaskResolver::new(&conn);
    let result = resolver.resolve(task_ref)?;
    let task = result.task;

    let derived = task.derive_status(&head_sha);
    let history = repo.get_proof_history(task.id)?;

    println!(
        "{} [{}] {}",
        status_icon(derived),
        task.slug.cyan().bold(),
        task.title
    );
    println!("   Status:  {} ({})", derived, derived.color_hint().dimmed());
    println!("   Repo:    {}", head_sha.dimmed());
    println!();

    print_explanation(derived, task.proof.as_ref(), &head_sha);
    println!();
    print_history(&history);

    Ok(())
}

fn status_icon(status: DerivedStatus) -> colored::ColoredString {
    match status {
        DerivedStatus::Proven => "✓".green(),
        DerivedStatus::Stale => "⚡".yellow(),
        DerivedStatus::Broken => "✗".red(),
        DerivedStatus::Unproven => "○".dimmed(),
        DerivedStatus::Attested => "!".blue(),
    }
}

fn print_explanation(status: DerivedStatus, proof: Option<&Proof>, head: &str) {
    match status {
        DerivedStatus::Unproven => {
            println!("{} No proof has ever been recorded for this task.", "reason:".yellow());
        }
        DerivedStatus::Proven => {
            if let Some(p) = proof {
                println!(
                    "{} Valid proof exists for SHA {}.",
                    "reason:".green(),
                    &p.git_sha[..7.min(p.git_sha.len())]
                );
            }
        }
        DerivedStatus::Stale => {
            if let Some(p) = proof {
                println!("{} Proof exists, but repo has moved.", "reason:".yellow());
                println!("         Proof SHA:   {}", &p.git_sha[..7.min(p.git_sha.len())]);
                println!("         Current SHA: {}", &head[..7.min(head.len())]);
            }
        }
        DerivedStatus::Broken => {
            println!("{} The last verification attempt failed.", "reason:".red());
        }
        DerivedStatus::Attested => {
            if let Some(p) = proof {
                let reason = p.attested_reason.as_deref().unwrap_or("Unknown");
                println!("{} Manually attested by human.", "reason:".blue());
                println!("         Note: \"{reason}\"");
            }
        }
    }
}

fn print_history(history: &[Proof]) {
    println!("{}", "Audit Log:".dimmed().underline());
    if history.is_empty() {
        println!("   (No history)");
        return;
    }

    for proof in history {
        let sha = &proof.git_sha[..7.min(proof.git_sha.len())];
        let status = if proof.attested_reason.is_some() {
            "ATTESTED".blue()
        } else if proof.exit_code == 0 {
            "PASS    ".green()
        } else {
            "FAIL    ".red()
        };

        println!(
            "   {}  {}  {}  {}",
            proof.timestamp.dimmed(),
            sha.yellow(),
            status,
            format!("{}ms", proof.duration_ms).dimmed()
        );
    }
}