use clap::Parser;

mod config;
mod error;
mod cli;
mod api;

use cli::{Cli, Commands};
use api::Api;
use error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let api = Api::new(config::SERVER_URL.to_string());

    match &cli.command {
        Commands::Agents => {
            cli::agents::list_agents(&api).await?;
        }

        Commands::Jobs => {
            cli::jobs::list_jobs(&api).await?;
        }

        Commands::Exec { agent, command } => {
            cli::exec::agent_exec(&api, agent, command).await?;
        }
    }
    
    Ok(())
}
