//! Handler for the `why` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::context::RepoContext;
use roadmap::engine::db::Db;
use roadmap::engine::repo::ProofRepo;
use roadmap::engine::resolver::TaskResolver;
use roadmap::engine::types::{DerivedStatus, Proof};

/// Explains the status of a task and shows its audit log.
///
/// # Errors
/// Returns error if task resolution or DB query fails.
pub fn handle(task_ref: &str) -> Result<()> {
    let conn = Db::connect()?;
    let proof_repo = ProofRepo::new(&conn);
    let context = RepoContext::new()?;
    let head_sha = context.head_sha();

    let resolver = TaskResolver::new(&conn);
    let result = resolver.resolve(task_ref)?;
    let task = result.task;

    let derived = task.derive_status(&context);
    let history = proof_repo.get_history(task.id)?;

    println!(
        "{} [{}] {}",
        status_icon(derived),
        task.slug.cyan().bold(),
        task.title
    );
    println!("   Status:  {} ({})", derived, derived.color_hint().dimmed());
    println!("   Repo:    {}", head_sha.dimmed());
    println!();

    print_explanation(derived, task.proof.as_ref(), head_sha);
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
        DerivedStatus::Stale => explain_stale(proof, head),
        DerivedStatus::Attested => explain_attested(proof),
        DerivedStatus::Proven => explain_proven(proof),
        DerivedStatus::Unproven => explain_unproven(),
        DerivedStatus::Broken => explain_broken(proof),
    }
}

fn explain_stale(proof: Option<&Proof>, head: &str) {
    if let Some(p) = proof {
        println!("{} Proof exists, but repo has moved.", "reason:".yellow());
        println!("         Proof SHA:   {}", &p.git_sha[..7.min(p.git_sha.len())]);
        println!("         Current SHA: {}", &head[..7.min(head.len())]);
    }
}

fn explain_attested(proof: Option<&Proof>) {
    if let Some(p) = proof {
        let reason = p.attested_reason.as_deref().unwrap_or("Unknown");
        println!("{} Manually attested by human.", "reason:".blue());
        println!("         Note: \"{reason}\"");
    }
}

fn explain_proven(proof: Option<&Proof>) {
    if let Some(p) = proof {
        println!(
            "{} Valid proof exists for SHA {}.",
            "reason:".green(),
            &p.git_sha[..7.min(p.git_sha.len())]
        );
    }
}

fn explain_unproven() {
    println!("{} No proof has ever been recorded for this task.", "reason:".yellow());
}

fn explain_broken(proof: Option<&Proof>) {
    println!("{} The last verification attempt failed.", "reason:".red());
    if let Some(p) = proof {
        if !p.stderr.is_empty() {
            println!("\n{}:", "stderr".red());
            for line in p.stderr.lines().take(5) {
                println!("  {}", line.dimmed());
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