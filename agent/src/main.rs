mod config;
mod init;
mod error;
mod run;

#[tokio::main]
async fn main() {
    let agent_id = init::get_id().await.unwrap();
    println!("{:?}",agent_id);
    run::run(agent_id).await;
}