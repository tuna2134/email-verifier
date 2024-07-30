use crate::db::verify as db;
use crate::server::result::AppResult;
use crate::utils::state::AppState;

use std::env;
use std::sync::Arc;

use axum::extract::{Query, State};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use twilight_http::request::AuditLogReason;
use twilight_http::Client as HttpClient;
use twilight_model::id::Id;
use url::Url;

static DISCORD_CLIENT_ID: Lazy<String> = Lazy::new(|| env::var("DISCORD_CLIENT_ID").unwrap());
static BASE_REDIRECT_URL: Lazy<String> = Lazy::new(|| env::var("BASE_REDIRECT_URL").unwrap());
static DISCORD_CLIENT_SECRET: Lazy<String> =
    Lazy::new(|| env::var("DISCORD_CLIENT_SECRET").unwrap());

pub async fn main_path() -> AppResult<String> {
    Ok("Hello, world!".to_string())
}

pub async fn get_oauth_url() -> AppResult<String> {
    let mut url = Url::parse("https://discord.com/oauth2/authorize")?;
    url.query_pairs_mut()
        .append_pair("client_id", &DISCORD_CLIENT_ID)
        .append_pair("redirect_uri", &format!("{}/discord", *BASE_REDIRECT_URL))
        .append_pair("response_type", "code")
        .append_pair("scope", "identify email");
    Ok(url.to_string())
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
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
    Query(query): Query<CallbackQuery>,
    State(state): State<Arc<AppState>>,
) -> AppResult<String> {
    let client = reqwest::Client::new();
    let response: DiscordTokenResponse = client
        .post("https://discord.com/api/v10/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", DISCORD_CLIENT_ID.clone()),
            ("client_secret", DISCORD_CLIENT_SECRET.clone()),
            ("grant_type", "authorization_code".to_string()),
            ("code", query.code),
            ("redirect_uri", format!("{}/discord", *BASE_REDIRECT_URL)),
        ])
        .send()
        .await?
        .json()
        .await?;
    tracing::debug!("{:?}", response);

    let http = HttpClient::new(format!("Bearer {}", response.access_token));
    let user = http.current_user().await?.model().await?;
    let (user_id, guild_id) = {
        let cache = state.cache.lock().await;
        let data = cache.get(&format!("auth:{}", query.state)).unwrap();
        if let Some((user_id, guild_id)) = data.split_once(':') {
            (user_id.to_string(), guild_id.to_string())
        } else {
            return Err(anyhow::anyhow!("Invalid data").into());
        }
    };
    let user_id = user_id.parse::<u64>().unwrap();
    if user.id.get() != user_id {
        return Err(anyhow::anyhow!("Invalid user").into());
    }
    let guild_id = guild_id.parse::<i64>().unwrap();
    if let Some((email_pattern, role_id)) = db::get_guild(&state.pool, guild_id).await? {
        if user.email.is_none() {
            return Err(anyhow::anyhow!("User has no email").into());
        }
        let pattern = Regex::new(&email_pattern)?;
        if !pattern.is_match(user.email.as_ref().unwrap()) {
            return Err(anyhow::anyhow!("Invalid email").into());
        }
        state
            .http
            .add_guild_member_role(
                Id::new(guild_id as u64),
                Id::new(user_id),
                Id::new(role_id as u64),
            )
            .reason("Verified email")?
            .await?;
    }

    Ok("ok".to_string())
}
