use crate::utils::state::AppState;

use std::sync::Arc;

use axum::{routing::get, Router};
use tokio::net::TcpListener;

mod result;
mod routes;

pub async fn run_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/auth", get(routes::auth::main_path))
        .route("/auth/oauth", get(routes::auth::get_oauth_url))
        .route("/auth/callback/discord", get(routes::auth::callback))
        .with_state(Arc::clone(&state));

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
