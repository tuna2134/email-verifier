use crate::db::mail_address as mail_db;
use crate::db::verify as db;
use crate::server::result::{APIError, APIResult};
use crate::utils::state::AppState;

use std::env;
use std::sync::Arc;

use axum::extract::{Json, State};
use bb8_redis::redis::AsyncCommands;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use twilight_http::request::AuditLogReason;
use twilight_http::Client as HttpClient;
use twilight_model::id::Id;
use twilight_model::user::CurrentUser;

static DISCORD_CLIENT_ID: Lazy<String> = Lazy::new(|| env::var("DISCORD_CLIENT_ID").unwrap());
static BASE_URL: Lazy<String> = Lazy::new(|| env::var("BASE_URL").unwrap());
static DISCORD_CLIENT_SECRET: Lazy<String> =
    Lazy::new(|| env::var("DISCORD_CLIENT_SECRET").unwrap());

pub async fn main_path() -> String {
    "Hello, world!".to_string()
}

#[derive(Deserialize)]
pub struct RequestVerifyDiscord {
    code: String,
    state: String,
}

#[derive(Serialize)]
pub struct ResponseVerifyDiscord {
    status: i32,
    user: CurrentUser,
}

#[derive(Deserialize, Debug)]
pub struct DiscordTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
}

pub async fn verify_discord(
    State(state): State<Arc<AppState>>,
    Json(query): Json<RequestVerifyDiscord>,
) -> APIResult<Json<ResponseVerifyDiscord>> {
    let client = reqwest::Client::new();
    let response: DiscordTokenResponse = client
        .post("https://discord.com/api/v10/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", DISCORD_CLIENT_ID.clone()),
            ("client_secret", DISCORD_CLIENT_SECRET.clone()),
            ("grant_type", "authorization_code".to_string()),
            ("code", query.code),
            (
                "redirect_uri",
                format!("{}/auth/callback/discord", *BASE_URL),
            ),
        ])
        .send()
        .await?
        .json()
        .await?;
    tracing::debug!("{:?}", response);

    let http = HttpClient::new(format!("Bearer {}", response.access_token));
    let user = http.current_user().await?.model().await?;
    let (user_id, guild_id) = {
        let mut conn = state.redis.get().await?;
        let data: String = conn.get(&format!("auth:{}", query.state)).await?;
        if let Some((user_id, guild_id)) = data.split_once(':') {
            (user_id.to_string(), guild_id.to_string())
        } else {
            return Err(APIError::badrequest("Invalid state"));
        }
    };
    let user_id = user_id.parse::<u64>().unwrap();
    if user.id.get() != user_id {
        return Err(APIError::badrequest("Invalid user"));
    }
    let guild_id = guild_id.parse::<i64>().unwrap();
    if let Some((email_pattern, role_id, _, enable_check_mail)) =
        db::get_guild(&state.pool, guild_id).await?
    {
        if user.email.is_none() {
            return Err(APIError::badrequest("Email not found"));
        }
        let pattern = Regex::new(&email_pattern)?;
        if !pattern.is_match(user.email.as_ref().unwrap()) {
            return Err(APIError::badrequest("Mail is not match"));
        }
        if enable_check_mail
            && !mail_db::exist_mail(
                &state.pool,
                guild_id,
                user.email.clone().unwrap_or("".to_string()),
            )
            .await?
        {
            return Err(APIError::badrequest("Mail is not inside at list"));
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
        {
            let mut conn = state.redis.get().await?;
            conn.del(&format!("auth:{}", query.state)).await?;
        }
    }

    Ok(Json(ResponseVerifyDiscord { status: 200, user }))
}
