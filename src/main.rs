mod handlers;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "roadmap", version, about = "Git for your Intent")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
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
        /// File glob patterns to scope this task (e.g., "src/auth/**")
        #[arg(long, short = 's')]
        scope: Option<Vec<String>>,
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
    /// Scan for invalidated (stale) proofs
    Stale,
    /// Show chronological verification history
    History {
        /// Number of entries to show
        #[arg(long, default_value = "20")]
        limit: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init | Commands::Add { .. } | Commands::Do { .. } | Commands::Check { .. } => {
            dispatch_write_ops(cli.command)
        }
        Commands::Next { .. }
        | Commands::List
        | Commands::Status
        | Commands::Why { .. }
        | Commands::Stale
        | Commands::History { .. } => dispatch_read_ops(cli.command),
    }
}

fn dispatch_write_ops(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Init => handlers::init::handle(),
        Commands::Add {
            title,
            blocks,
            after,
            test,
            scope,
        } => handlers::add::handle(
            &title,
            blocks.as_deref(),
            after.as_deref(),
            test.as_deref(),
            scope,
        ),
        Commands::Do { task, strict } => handlers::do_task::handle(&task, strict),
        Commands::Check { force, reason } => handlers::check::handle(force, reason.as_deref()),
        _ => unreachable!("Invalid write command dispatch"),
    }
}

fn dispatch_read_ops(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Next { json } => handlers::next::handle(json),
        Commands::List => handlers::list::handle(),
        Commands::Status => handlers::status::handle(),
        Commands::Why { task } => handlers::why::handle(&task),
        Commands::Stale => handlers::stale::handle(),
        Commands::History { limit } => handlers::history::handle(limit),
        _ => unreachable!("Invalid read command dispatch"),
    }
}