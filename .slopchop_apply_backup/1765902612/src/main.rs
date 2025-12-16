use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;

#[derive(Parser)]
#[command(name = "roadmap", version, about = "Git for your Intent", long_about = None)]
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
        /// Task ID this new task blocks (is a dependency of)
        #[arg(long, short = 'b')]
        blocks: Option<String>,
        /// Task ID that blocks this new task (is a prerequisite)
        #[arg(long, short = 'a')]
        after: Option<String>,
    },
    /// List next actionable tasks (Topological Sort)
    Next,
    /// List all tasks
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            Db::init()?;
            println!("{} Initialized .roadmap/state.db", "✓".green());
        }
        Commands::Add {
            title,
            blocks,
            after,
        } => {
            let _conn = Db::connect()?;
            // Placeholder logic until full graph engine is ready
            println!("{} Adding task: {title}", "➜".cyan());
            if let Some(b) = blocks {
                println!("   Blocks: {b}");
            }
            if let Some(a) = after {
                println!("   After:  {a}");
            }
            // Logic to insert into DB goes here
        }
        Commands::Next => {
            let conn = Db::connect()?;
            let graph = TaskGraph::build(&conn)?;
            let critical_path = graph.get_critical_path();

            println!("{} Next Actionable Tasks:", "➜".cyan());
            if critical_path.is_empty() {
                println!("   (No pending tasks found)");
            } else {
                for task in critical_path {
                    println!("   [{}] {}", task.slug.yellow(), task.title);
                }
            }
        }
        Commands::List => {
            let conn = Db::connect()?;
            let repo = TaskRepo::new(conn);
            let tasks = repo.get_all()?;

            println!("{} All Tasks:", "➜".cyan());
            for task in tasks {
                println!("   [{}] {} ({})", task.slug.blue(), task.title, task.status);
            }
        }
    }

    Ok(())
}