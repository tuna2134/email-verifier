use crate::utils::state::AppState;

use std::sync::Arc;
use std::env;

use axum::{routing::{get, post}, Router, http::{HeaderValue, Method}};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod result;
mod routes;

pub async fn run_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let allow_origin = env::var("ALLOW_ORIGIN")?;
    let app = Router::new()
        .route("/auth", get(routes::auth::main_path))
        .route("/auth/verify/discord", post(routes::auth::verify_discord))
        .layer(
            CorsLayer::new()
                .allow_origin(allow_origin.parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST]),
        )
        .with_state(Arc::clone(&state));

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
