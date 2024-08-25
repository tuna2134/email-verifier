use crate::db::token as db;
use crate::server::result::AppResult;
use crate::server::token::Token;
use crate::utils::state::AppState;

use std::env;
use std::sync::Arc;

use axum::extract::{Json, State};
use base64::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use twilight_http::Client as HttpClient;

static DISCORD_CLIENT_ID: Lazy<String> = Lazy::new(|| env::var("DISCORD_CLIENT_ID").unwrap());
static BASE_URL: Lazy<String> = Lazy::new(|| env::var("BASE_URL").unwrap());
static DISCORD_CLIENT_SECRET: Lazy<String> =
    Lazy::new(|| env::var("DISCORD_CLIENT_SECRET").unwrap());

#[derive(Deserialize)]
pub struct RequestDashboardCallback {
    code: String,
    state: String,
}

#[derive(Serialize)]
pub struct ResponseDashboardCallback {
    status: i32,
    token: String,
}

#[derive(Deserialize, Debug)]
pub struct DiscordTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
}

pub async fn callback(
    State(state): State<Arc<AppState>>,
    Json(query): Json<RequestDashboardCallback>,
) -> AppResult<Json<ResponseDashboardCallback>> {
    let client = reqwest::Client::new();
    let response: DiscordTokenResponse = client
        .post("https://discord.com/api/v10/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", DISCORD_CLIENT_ID.clone()),
            ("client_secret", DISCORD_CLIENT_SECRET.clone()),
            ("grant_type", "authorization_code".to_string()),
            ("code", query.code),
            ("redirect_uri", format!("{}/dashboard/callback", *BASE_URL)),
        ])
        .send()
        .await?
        .json()
        .await?;
    tracing::info!("{:?}", response);

    let http = HttpClient::new(format!("Bearer {}", response.access_token));
    let user = http.current_user().await?.model().await?;

    let token = Token::new(user.id.get())?;
    let nonce = BASE64_URL_SAFE_NO_PAD.encode(&token.nonce);
    db::set_token(&state.pool, user.id.get() as i64, nonce).await?;

    Ok(Json(ResponseDashboardCallback {
        status: 200,
        token: token.generate()?,
    }))
}
