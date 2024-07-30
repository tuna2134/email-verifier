use crate::server::result::AppResult;

use std::env;

use axum::extract::Query;
use once_cell::sync::Lazy;
use serde::Deserialize;
use twilight_http::Client as HttpClient;
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

pub async fn callback(Query(query): Query<CallbackQuery>) -> AppResult<String> {
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
    tracing::debug!("{:?}", user);

    Ok("ok".to_string())
}
