use std::env;

use once_cell::sync::Lazy;
use url::Url;

pub mod state;

static DISCORD_CLIENT_ID: Lazy<String> = Lazy::new(|| env::var("DISCORD_CLIENT_ID").unwrap());
static BASE_REDIRECT_URL: Lazy<String> = Lazy::new(|| env::var("BASE_REDIRECT_URL").unwrap());

pub async fn get_oauth_url(state: String) -> anyhow::Result<String> {
    let mut url = Url::parse("https://discord.com/oauth2/authorize")?;
    url.query_pairs_mut()
        .append_pair("client_id", &DISCORD_CLIENT_ID)
        .append_pair("redirect_uri", &format!("{}/discord", *BASE_REDIRECT_URL))
        .append_pair("response_type", "code")
        .append_pair("scope", "identify email")
        .append_pair("state", &state);
    Ok(url.to_string())
}
