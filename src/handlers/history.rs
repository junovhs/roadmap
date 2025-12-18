//! Handler for the `history` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::repo::ProofRepo;

/// Displays the global verification history.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle(limit: usize) -> Result<()> {
    let conn = Db::connect()?;
    let proof_repo = ProofRepo::new(&conn);
    
    let history = proof_repo.get_global_history(limit)?;

    println!("{} Project History (last {})", "📜".cyan(), limit);
    println!();

    if history.is_empty() {
        println!("   (No history recorded yet)");
        return Ok(());
    }

    for (slug, proof) in history {
        let timestamp = &proof.timestamp[..19.min(proof.timestamp.len())].replace('T', " ");
        
        let status = if proof.attested_reason.is_some() {
            "ATTESTED".blue()
        } else if proof.exit_code == 0 {
            "PASS    ".green()
        } else {
            "FAIL    ".red()
        };

        println!(
            "   {}  {}  {}  {}",
            timestamp.dimmed(),
            status,
            slug.bold(),
            format!("{}ms", proof.duration_ms).dimmed()
        );
    }

    Ok(())
}