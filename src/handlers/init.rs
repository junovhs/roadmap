//! Handler for the `init` command.

use anyhow::Result;
use colored::Colorize;
use roadmap::engine::db::Db;

/// Initializes the roadmap repository.
///
/// # Errors
/// Returns error if database initialization fails.
pub fn handle() -> Result<()> {
    Db::init()?;
    println!("{} Initialized .roadmap/state.db", "✓".green());
    Ok(())
}