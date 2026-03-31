mod cli;
mod job;
mod system;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { schedule, command, id, desc, source, force } => {
            println!("add: schedule={schedule}, command={command}");
        }
        Commands::Remove { identifier } => {
            println!("remove: {identifier}");
        }
        Commands::List { json } => {
            println!("list: json={json}");
        }
        Commands::Search { query, json } => {
            println!("search: query={query}, json={json}");
        }
        Commands::Enable { identifier } => {
            println!("enable: {identifier}");
        }
        Commands::Disable { identifier } => {
            println!("disable: {identifier}");
        }
        Commands::RunOnce { id, cleanup_id, command } => {
            println!("run-once: id={id}, cleanup_id={cleanup_id}");
        }
    }

    Ok(())
}
