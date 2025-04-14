use clap::Parser;

mod api;
mod cli;
mod config;
mod error;

use api::Api;
use cli::{Cli, Commands};
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
            let config = config::Config::load()?;
            cli::exec::agent_exec(&api, agent, command, config).await?;
        }

        Commands::Signing => {
            cli::signing::generate_keypair();
        }
    }

    Ok(())
}
