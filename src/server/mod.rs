use crate::utils::state::AppState;

use std::env;
use std::sync::Arc;

use axum::{
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod result;
mod routes;
mod token;

pub async fn run_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let allow_origin = env::var("BASE_URL")?;
    let app = Router::new()
        .route("/auth", get(routes::auth::main_path))
        .route("/auth/verify/discord", post(routes::auth::verify_discord))
        .route(
            "/dashboard/exchange_token",
            post(routes::dashboard::callback),
        )
        .route("/dashboard/users/@me", get(routes::dashboard::get_me))
        .route(
            "/dashboard/users/@me/guilds",
            get(routes::dashboard::get_me_guilds),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(allow_origin.parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any),
        )
        .with_state(Arc::clone(&state));

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
