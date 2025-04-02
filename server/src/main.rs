use std::sync::Arc;

mod api;
mod config;
mod db;
mod entities;
mod error;
mod repository;
mod service;
mod share;

use api::routes::routes;
use api::state::AppState;
use config::Config;
use service::Service;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var(
        "DATABASE_URL",
        "postgres://postgres:root@127.0.0.1:5432/server",
    );

    env_logger::init();

    let config = Config::load()?;

    let db_pool = db::connect(&config.database_url).await?;
    db::migrare(&db_pool).await?;

    let service = Service::new(db_pool);
    let app_state = Arc::new(AppState::new(service));

    let routes = routes(app_state);

    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], config.port), async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for CRTL+c");
            log::info!("Shutting down server");
        });
    log::info!("starting on: {:?}", addr);
    server.await;

    Ok(())
}
