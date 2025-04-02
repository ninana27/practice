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

    let agent_config = init::get_config(&client).await?;
    println!("{:?}",agent_config);
    //run::run(&client, agent_config).await;
    Ok(())
}