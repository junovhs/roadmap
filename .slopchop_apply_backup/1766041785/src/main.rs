mod handlers;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use roadmap::engine::db::Db;

#[derive(Parser)]
#[command(name = "roadmap", version, about = "Git for your Intent")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the roadmap repository
    Init,
    /// Add a new task
    Add {
        title: String,
        #[arg(long, short = 'b')]
        blocks: Option<String>,
        #[arg(long, short = 'a')]
        after: Option<String>,
        #[arg(long, short = 't')]
        test: Option<String>,
    },
    /// Show next actionable tasks
    Next {
        #[arg(long)]
        json: bool,
    },
    /// List all tasks
    List,
    /// Set active task
    Do {
        task: String,
        /// Strict mode: require exact ID or slug (no fuzzy matching)
        #[arg(long)]
        strict: bool,
    },
    /// Run verification for active task
    Check {
        /// Mark complete without verification (creates ATTESTED, not DONE)
        #[arg(long)]
        force: bool,
        /// Reason for manual attestation (required with --force)
        #[arg(long, requires = "force")]
        reason: Option<String>,
    },
    /// Show current status
    Status,
    /// Explain the status of a specific task
    Why {
        task: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            Db::init()?;
            println!("{} Initialized .roadmap/state.db", "✓".green());
            Ok(())
        }
        Commands::Add { title, blocks, after, test } => {
            handlers::add::handle(title, blocks.as_deref(), after.as_deref(), test.as_deref())
        }
        Commands::Next { json } => handlers::next::handle(*json),
        Commands::List => handlers::list::handle(),
        Commands::Do { task, strict } => handlers::do_task::handle(task, *strict),
        Commands::Check { force, reason } => handlers::check::handle(*force, reason.as_deref()),
        Commands::Status => handlers::status::handle(),
        Commands::Why { task } => handlers::why::handle(task),
    }
}