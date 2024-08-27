use utils::state::AppState;

use std::{env, sync::Arc};

mod bot;
mod db;
mod server;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN")?;

    let state = Arc::new(
        AppState::setup(
            env::var("DATABASE_URL")?,
            token.clone(),
            env::var("REDIS_URL")?,
        )
        .await?,
    );

    tokio::spawn(bot::run_bot(Arc::clone(&state), token));

    server::run_server(Arc::clone(&state)).await?;
    Ok(())
}
