use clap::{Parser, Subcommand};

pub mod agents;
pub mod jobs;
pub mod exec;

/// A client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(after_long_help = "See 'client help <command>' for more information on a specific command.")]
pub struct  Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {

    /// List all agents
    Agents,

    /// List all job
    Jobs,

    /// Execute a command to the agent by agent id 
    Exec {

        /// The agent id to execute the command on
        #[arg(short, long)]
        agent: String,

        /// The command to execute, with its arguments
        #[arg(long)]
        command: String,
    }
}