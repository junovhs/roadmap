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
    Do { task: String },
    /// Run verification for active task
    Check,
    /// Show current status
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            Db::init()?;
            println!("{} Initialized .roadmap/state.db", "�".green());
            Ok(())
        }
        Commands::Add { title, blocks, after, test } => {
            handlers::add::handle(title, blocks.as_deref(), after.as_deref(), test.as_deref())
        }
        Commands::Next { json } => handlers::next::handle(*json),
        Commands::List => handlers::list::handle(),
        Commands::Do { task } => handlers::do_task::handle(task),
        Commands::Check => handlers::check::handle(),
        Commands::Status => handlers::status::handle(),
    }
}