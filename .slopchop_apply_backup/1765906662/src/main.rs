use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use roadmap::engine::db::Db;
use roadmap::engine::graph::TaskGraph;
use roadmap::engine::repo::TaskRepo;
use roadmap::engine::resolver::{slugify, TaskResolver};
use roadmap::engine::runner::VerifyRunner;
use roadmap::engine::types::TaskStatus;

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
        /// Task title or slug
        title: String,

        /// Task this new task blocks (is a prerequisite of)
        #[arg(long, short = 'b')]
        blocks: Option<String>,

        /// Task that blocks this new task (prerequisite)
        #[arg(long, short = 'a')]
        after: Option<String>,

        /// Verification command (e.g., "cargo test foo")
        #[arg(long, short = 't')]
        test: Option<String>,
    },

    /// Show next actionable tasks (Critical Path)
    Next {
        /// Output as JSON for agent consumption
        #[arg(long)]
        json: bool,
    },

    /// List all tasks
    List,

    /// Set active task ("I am working on this")
    Do {
        /// Task identifier (ID, slug, or fuzzy name)
        task: String,
    },

    /// Run verification for active task
    Check,

    /// Show current status
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => handle_init(),
        Commands::Add {
            title,
            blocks,
            after,
            test,
        } => handle_add(title, blocks.as_deref(), after.as_deref(), test.as_deref()),
        Commands::Next { json } => handle_next(*json),
        Commands::List => handle_list(),
        Commands::Do { task } => handle_do(task),
        Commands::Check => handle_check(),
        Commands::Status => handle_status(),
    }
}

fn handle_init() -> Result<()> {
    Db::init()?;
    println!("{} Initialized .roadmap/state.db", "✓".green());
    Ok(())
}

fn handle_add(
    title: &str,
    blocks: Option<&str>,
    after: Option<&str>,
    test_cmd: Option<&str>,
) -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);

    let slug = slugify(title);

    // Check for duplicate slug
    if repo.find_by_slug(&slug)?.is_some() {
        bail!("Task with slug '{slug}' already exists");
    }

    // Insert the task
    let task_id = if let Some(cmd) = test_cmd {
        repo.add_with_test(&slug, title, cmd)?
    } else {
        repo.add(&slug, title)?
    };

    println!("{} Added task [{}] {}", "✓".green(), slug.yellow(), title);

    // Handle dependencies
    let resolver = TaskResolver::new(repo.conn());

    // If --after is specified, create edge: after_task -> new_task
    if let Some(after_ref) = after {
        let after_task = resolver.resolve(after_ref)?;

        // Check for cycles
        let graph = TaskGraph::build(repo.conn())?;
        if graph.would_create_cycle(after_task.task.id, task_id) {
            bail!("Adding this dependency would create a cycle!");
        }

        repo.link(after_task.task.id, task_id)?;
        println!(
            "   {} [{}] blocks [{}]",
            "→".cyan(),
            after_task.task.slug,
            slug
        );
    }

    // If --blocks is specified, create edge: new_task -> blocks_task
    if let Some(blocks_ref) = blocks {
        let blocks_task = resolver.resolve(blocks_ref)?;

        // Check for cycles
        let graph = TaskGraph::build(repo.conn())?;
        if graph.would_create_cycle(task_id, blocks_task.task.id) {
            bail!("Adding this dependency would create a cycle!");
        }

        repo.link(task_id, blocks_task.task.id)?;
        println!(
            "   {} [{}] blocks [{}]",
            "→".cyan(),
            slug,
            blocks_task.task.slug
        );
    }

    Ok(())
}

fn handle_next(json: bool) -> Result<()> {
    let conn = Db::connect()?;
    let graph = TaskGraph::build(&conn)?;
    let critical_path = graph.get_critical_path();

    if json {
        let tasks: Vec<_> = critical_path
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "slug": t.slug,
                    "title": t.title,
                    "status": t.status.to_string(),
                    "test_cmd": t.test_cmd
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&tasks)?);
        return Ok(());
    }

    println!("{} Next Actionable Tasks:", "🎯".to_string().cyan());

    if critical_path.is_empty() {
        println!(
            "   {} All tasks completed or none defined.",
            "(empty)".dimmed()
        );
        return Ok(());
    }

    for task in critical_path {
        let status_icon = match task.status {
            TaskStatus::Pending => "○".dimmed(),
            TaskStatus::Active => "●".yellow(),
            TaskStatus::Done => "✓".green(),
            TaskStatus::Blocked => "⊘".red(),
        };

        println!("   {} [{}] {}", status_icon, task.slug.yellow(), task.title);

        // Show what this task blocks
        let blocked = graph.get_blocked_by(task.id);
        if !blocked.is_empty() {
            let names: Vec<_> = blocked.iter().map(|t| t.slug.as_str()).collect();
            println!(
                "      {} {}",
                "└─ blocks:".dimmed(),
                names.join(", ").dimmed()
            );
        }
    }

    Ok(())
}

fn handle_list() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);
    let tasks = repo.get_all()?;

    println!("{} All Tasks:", "📋".to_string().cyan());

    if tasks.is_empty() {
        println!("   {} No tasks defined yet.", "(empty)".dimmed());
        return Ok(());
    }

    for task in tasks {
        let status_icon = match task.status {
            TaskStatus::Pending => "○".dimmed(),
            TaskStatus::Active => "●".yellow(),
            TaskStatus::Done => "✓".green(),
            TaskStatus::Blocked => "⊘".red(),
        };

        let test_indicator = if task.test_cmd.is_some() { " 🧪" } else { "" };

        println!(
            "   {} [{}] {} ({}){}",
            status_icon,
            task.slug.blue(),
            task.title,
            task.status.to_string().dimmed(),
            test_indicator
        );
    }

    Ok(())
}

fn handle_do(task_ref: &str) -> Result<()> {
    let conn = Db::connect()?;
    let resolver = TaskResolver::new(&conn);

    let result = resolver.resolve(task_ref)?;
    let task = &result.task;

    // Check if task is blocked
    let graph = TaskGraph::build(&conn)?;
    let blockers = graph.get_blockers(task.id);
    let active_blockers: Vec<_> = blockers
        .iter()
        .filter(|t| t.status != TaskStatus::Done)
        .collect();

    if !active_blockers.is_empty() {
        let names: Vec<_> = active_blockers.iter().map(|t| t.slug.as_str()).collect();
        bail!("Task [{}] is blocked by: {}", task.slug, names.join(", "));
    }

    // Set as active
    let repo = TaskRepo::new(conn);
    repo.update_status(task.id, TaskStatus::Active)?;
    repo.set_active_task(task.id)?;

    println!(
        "{} Now working on: [{}] {}",
        "●".yellow(),
        task.slug.yellow(),
        task.title
    );

    if let Some(ref cmd) = task.test_cmd {
        println!("   {} {}", "verify:".dimmed(), cmd.dimmed());
    }

    Ok(())
}

fn handle_check() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);

    // Get active task
    let active_id = repo.get_active_task_id()?;
    let active_id = match active_id {
        Some(id) => id,
        None => bail!("No active task. Run `roadmap do <task>` first."),
    };

    let task = repo.find_by_id(active_id)?;
    let task = match task {
        Some(t) => t,
        None => bail!("Active task not found in database."),
    };

    println!(
        "{} Checking: [{}] {}",
        "🔍".to_string(),
        task.slug.yellow(),
        task.title
    );

    // Get verification command
    let test_cmd = match &task.test_cmd {
        Some(cmd) => cmd,
        None => {
            // No test command - allow manual completion
            println!("{} No verification command defined.", "⚠".yellow());
            println!("   Run with --force to mark complete, or add a test:");
            println!("   roadmap edit {} --test \"your_test_cmd\"", task.slug);
            return Ok(());
        }
    };

    println!("   {} {}", "running:".dimmed(), test_cmd);

    // Execute verification
    let runner = VerifyRunner::default_runner();
    let result = runner.verify(test_cmd)?;

    if result.passed() {
        // Mark as DONE
        repo.update_status(task.id, TaskStatus::Done)?;

        println!(
            "{} Verified! Task [{}] marked DONE ({:.2}s)",
            "✓".green(),
            task.slug.green(),
            result.duration.as_secs_f64()
        );

        // Show what's now unblocked
        let graph = TaskGraph::build(repo.conn())?;
        let unblocked = graph.get_critical_path();
        let newly_available: Vec<_> = unblocked.iter().filter(|t| t.id != task.id).collect();

        if !newly_available.is_empty() {
            println!("\n{} Now available:", "🎯".to_string());
            for t in newly_available.iter().take(3) {
                println!("   ○ [{}] {}", t.slug.yellow(), t.title);
            }
        }
    } else {
        println!(
            "{} Verification failed. Task remains {}.",
            "✗".red(),
            "ACTIVE".yellow()
        );
    }

    Ok(())
}

fn handle_status() -> Result<()> {
    let conn = Db::connect()?;
    let repo = TaskRepo::new(conn);
    let graph = TaskGraph::build(repo.conn())?;

    let all_tasks = repo.get_all()?;
    let done_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();
    let total = all_tasks.len();

    println!("{} Roadmap Status", "📊".to_string().cyan());
    println!("   Tasks: {}/{} complete", done_count, total);
    println!(
        "   Graph: {} nodes, {} edges",
        graph.task_count(),
        graph.edge_count()
    );

    // Current focus
    if let Some(active_id) = repo.get_active_task_id()? {
        if let Some(task) = repo.find_by_id(active_id)? {
            println!(
                "\n{} Focus: [{}] {}",
                "●".yellow(),
                task.slug.yellow(),
                task.title
            );
        }
    } else {
        println!("\n{} No active task", "○".dimmed());
    }

    // Next up
    let critical = graph.get_critical_path();
    if !critical.is_empty() {
        println!("\n{} Next up:", "→".cyan());
        for task in critical.iter().take(3) {
            println!("   ○ [{}] {}", task.slug.dimmed(), task.title);
        }
    }

    Ok(())
}
