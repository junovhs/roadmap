//! Handler for the `history` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::repo::ProofRepo;
use roadmap::engine::types::Proof;
use serde::Serialize;

/// Displays the global verification history.
///
/// # Errors
/// Returns error if database query fails.
pub fn handle(limit: usize, json: bool) -> Result<()> {
    let conn = Db::connect()?;
    let proof_repo = ProofRepo::new(&conn);
    
    let history = proof_repo.get_global_history(limit)?;

    if json {
        return print_json(&history);
    }

    print_human(&history, limit);
    Ok(())
}

#[derive(Serialize)]
struct HistoryEntry {
    slug: String,
    proof: Proof,
}

fn print_json(history: &[(String, Proof)]) -> Result<()> {
    let entries: Vec<HistoryEntry> = history.iter().map(|(slug, proof)| {
        HistoryEntry {
            slug: slug.clone(),
            proof: proof.clone(),
        }
    }).collect();
    println!("{}", serde_json::to_string_pretty(&entries)?);
    Ok(())
}

fn print_human(history: &[(String, Proof)], limit: usize) {
    println!("{} Project History (last {})", "📜".cyan(), limit);
    println!();

    if history.is_empty() {
        println!("   (No history recorded yet)");
        return;
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
}