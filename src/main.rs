mod cli;
mod commands;
mod job;
mod output;
mod sidecar;
mod system;
mod schedule;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { schedule, command, id, desc, source, force } => {
            commands::add::run(schedule, command, id, desc, source, force)?;
        }
        Commands::Remove { identifier } => {
            commands::remove::run(&identifier)?;
        }
        Commands::List { json } => {
            commands::list::run(json)?;
        }
        Commands::Search { query, json } => {
            println!("search: query={query}, json={json}");
        }
        Commands::Enable { identifier } => {
            commands::enable_disable::run_enable(&identifier)?;
        }
        Commands::Disable { identifier } => {
            commands::enable_disable::run_disable(&identifier)?;
        }
        Commands::RunOnce { id, cleanup_id, command } => {
            println!("run-once: id={id}, cleanup_id={cleanup_id}");
        }
    }

    Ok(())
}
