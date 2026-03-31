use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "chronos", about = "CLI for managing cron jobs and scheduled tasks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a scheduled job
    Add {
        /// Schedule expression (cron or natural language)
        schedule: String,
        /// Command to execute
        command: String,
        /// Human-friendly job identifier
        #[arg(long)]
        id: Option<String>,
        /// Job description
        #[arg(long)]
        desc: Option<String>,
        /// Who scheduled this job (defaults to $USER)
        #[arg(long)]
        source: Option<String>,
        /// Skip duplicate warning
        #[arg(long)]
        force: bool,
    },
    /// Remove a scheduled job
    Remove {
        /// Job identifier (sidecar id or row number)
        identifier: String,
    },
    /// List all scheduled jobs
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Search for jobs by query
    Search {
        /// Search query
        query: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Enable a disabled job
    Enable {
        /// Job identifier (sidecar id or row number)
        identifier: String,
    },
    /// Disable a job (comments it out)
    Disable {
        /// Job identifier (sidecar id or row number)
        identifier: String,
    },
    /// Internal: run a one-off job and self-remove
    #[command(name = "_run-once", hide = true)]
    RunOnce {
        /// Job ID to remove after execution
        id: String,
        /// Cleanup job ID to also remove
        #[arg(long)]
        cleanup_id: String,
        /// Command separator
        #[arg(last = true)]
        command: Vec<String>,
    },
}
