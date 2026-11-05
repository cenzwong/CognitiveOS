use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cognitive_core::ids::ProcessId;
use cognitive_kernel::KernelService;
use cognitive_persistence::sqlite::{create_sqlite_pool, run_migrations};
use cognitive_persistence::SqliteProcessRepository;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    name = "cog",
    author,
    version,
    about = "CognitiveOS Process Control Interface",
    long_about = None
)]
struct Cli {
    /// Path to SQLite database file
    #[arg(long, global = true, env = "COG_DB")]
    db: Option<PathBuf>,

    /// Output format as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create and admit a process to the ready pool
    Run {
        /// The objective of the cognitive process
        objective: String,

        /// Path to the workspace for this process
        #[arg(long, default_value = ".")]
        workspace: String,
    },

    /// List all processes
    Ps,

    /// Inspect a process in detail
    Inspect {
        /// The Process ID (e.g., proc-...)
        process_id: String,
    },

    /// Pause an active process
    Pause {
        /// The Process ID to pause
        process_id: String,

        /// Optional reason for pausing
        #[arg(long)]
        reason: Option<String>,
    },

    /// Resume a paused process
    Resume {
        /// The Process ID to resume
        process_id: String,

        /// Optional reason for resuming
        #[arg(long)]
        reason: Option<String>,
    },

    /// Cancel a process
    Cancel {
        /// The Process ID to cancel
        process_id: String,

        /// Optional reason for cancellation
        #[arg(long)]
        reason: Option<String>,
    },
}

fn resolve_db_path(db_arg: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = db_arg {
        return Ok(p);
    }

    if let Ok(env_path) = std::env::var("COG_DB") {
        return Ok(PathBuf::from(env_path));
    }

    let base_dir = dirs_or_fallback();
    let app_dir = base_dir.join("cognitiveos");
    std::fs::create_dir_all(&app_dir)
        .with_context(|| format!("Failed to create data directory at {}", app_dir.display()))?;
    Ok(app_dir.join("cognitiveos.db"))
}

fn dirs_or_fallback() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        std::env::temp_dir()
    }
}

async fn initialize_kernel(db_path: &Path) -> Result<KernelService<SqliteProcessRepository>> {
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = create_sqlite_pool(&db_url)
        .await
        .with_context(|| format!("Failed to connect to SQLite at {}", db_url))?;

    run_migrations(&pool)
        .await
        .with_context(|| "Failed to run database migrations")?;

    let repo = Arc::new(SqliteProcessRepository::new(pool));
    Ok(KernelService::new(repo))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = resolve_db_path(cli.db)?;
    let kernel = initialize_kernel(&db_path).await?;

    match cli.command {
        Commands::Run {
            objective,
            workspace,
        } => {
            let process = kernel.run_process(objective, workspace).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&process)?);
            } else {
                println!("Process created and admitted to READY pool.");
                println!(
                    "Notice: Native model inference execution has not been started (Phase 1 kernel mode)."
                );
                println!("Process ID:      {}", process.process_id);
                println!("State:           {}", process.state);
                println!("State Version:   {}", process.state_version);
                println!("Objective:       {}", process.objective);
                println!("Workspace:       {}", process.workspace_id);
            }
        }

        Commands::Ps => {
            let processes = kernel.list_processes().await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&processes)?);
            } else if processes.is_empty() {
                println!("No processes found.");
            } else {
                println!(
                    "{:<36} {:<12} {:<8} {:<}",
                    "PROCESS ID", "STATE", "VERSION", "OBJECTIVE"
                );
                println!("{}", "-".repeat(80));
                for p in processes {
                    let truncated_obj = if p.objective.len() > 30 {
                        format!("{}...", &p.objective[..27])
                    } else {
                        p.objective
                    };
                    println!(
                        "{:<36} {:<12} {:<8} {:<}",
                        p.process_id, p.state, p.state_version, truncated_obj
                    );
                }
            }
        }

        Commands::Inspect { process_id } => {
            let proc_id = ProcessId::from(process_id);
            let process = kernel.get_process(&proc_id).await?;
            let events = kernel.get_process_events(&proc_id, None).await?;

            if cli.json {
                let out = serde_json::json!({
                    "process": process,
                    "events": events,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("=== PROCESS DETAILS ===");
                println!("Process ID:      {}", process.process_id);
                println!("Objective:       {}", process.objective);
                println!("State:           {}", process.state);
                println!("State Version:   {}", process.state_version);
                println!("Workspace:       {}", process.workspace_id);
                println!("Program ID:      {}", process.program_id);
                println!("Model Profile:   {}", process.model_profile_id);
                println!("Created At:      {}", process.created_at);
                println!("Updated At:      {}", process.updated_at);
                if let Some(completed_at) = process.completed_at {
                    println!("Completed At:    {}", completed_at);
                }

                println!(
                    "
=== EVENT JOURNAL (Monotonic Sequence) ==="
                );
                for ev in events {
                    println!(
                        "#{:<3} [{}] {} ({})",
                        ev.sequence,
                        ev.event_type.as_str(),
                        ev.timestamp,
                        ev.event_id
                    );
                }
            }
        }

        Commands::Pause { process_id, reason } => {
            let proc_id = ProcessId::from(process_id);
            let process = kernel.pause_process(&proc_id, reason).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&process)?);
            } else {
                println!(
                    "Process {} paused (version {}).",
                    process.process_id, process.state_version
                );
            }
        }

        Commands::Resume { process_id, reason } => {
            let proc_id = ProcessId::from(process_id);
            let process = kernel.resume_process(&proc_id, reason).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&process)?);
            } else {
                println!(
                    "Process {} resumed to READY (version {}).",
                    process.process_id, process.state_version
                );
            }
        }

        Commands::Cancel { process_id, reason } => {
            let proc_id = ProcessId::from(process_id);
            let process = kernel.cancel_process(&proc_id, reason).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&process)?);
            } else {
                println!(
                    "Process {} cancelled (version {}).",
                    process.process_id, process.state_version
                );
            }
        }
    }

    Ok(())
}
