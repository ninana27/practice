use reqwest;
use std::time::Duration;

mod config;
mod init;
mod error;
mod run;

use error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let timeout = Duration::from_secs(5);
    let client = reqwest::Client::builder()
        .connect_timeout(timeout)
        .timeout(timeout)
        .build()?;

    let agent_id = init::get_id(&client).await?;
    println!("{:?}",agent_id);
    run::run(&client, agent_id).await;
}