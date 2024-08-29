use crate::db::mail_address as mail_db;
use crate::db::token as db;
use crate::db::verify as verify_db;
use crate::server::result::{APIError, APIResult};
use crate::server::token::Token;
use crate::utils::state::AppState;

use std::env;
use std::sync::Arc;

use axum::extract::{Json, Path, State};
use base64::prelude::*;
use bb8_redis::redis::AsyncCommands;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sparkle_interactions::builder::component::{ButtonBuilder, ComponentsBuilder};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::channel::{Channel, ChannelType};
use twilight_model::guild::Guild;
use twilight_model::guild::Permissions;
use twilight_model::guild::Role;
use twilight_model::id::Id;
use twilight_model::user::{CurrentUser, CurrentUserGuild};
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::permission_calculator::PermissionCalculator;

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

#[derive(Deserialize)]
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
) -> APIResult<Json<ResponseDashboardCallback>> {
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

    let http = HttpClient::new(format!("Bearer {}", response.access_token));
    let user = http.current_user().await?.model().await?;

    let token = Token::new(user.id.get())?;
    let nonce = BASE64_URL_SAFE_NO_PAD.encode(token.nonce);
    db::set_token(
        &state.pool,
        user.id.get() as i64,
        nonce,
        response.access_token,
    )
    .await?;

    {
        let mut conn = state.redis.get().await?;
        conn.set_ex::<_, _, ()>(
            format!("dashboard:user:{}", user.id),
            serde_json::to_string(&user)?,
            60,
        )
        .await?;
    }

    Ok(Json(ResponseDashboardCallback {
        status: 200,
        token: token.generate()?,
    }))
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    token: Token,
) -> APIResult<Json<CurrentUser>> {
    let user = {
        let mut conn = state.redis.get().await?;
        let data: Option<String> = conn
            .get(&format!("dashboard:user:{}", token.user_id))
            .await?;
        if let Some(data) = data {
            serde_json::from_str(&data)?
        } else {
            let access_token = db::get_access_token(&state.pool, token.user_id as i64).await?;
            let http = HttpClient::new(format!("Bearer {}", access_token));
            let user = http.current_user().await?.model().await?;
            conn.set_ex::<_, _, ()>(
                format!("dashboard:user:{}", token.user_id),
                serde_json::to_string(&user)?,
                60,
            )
            .await?;
            user
        }
    };

    Ok(Json(user))
}

pub async fn get_me_guilds(
    State(state): State<Arc<AppState>>,
    token: Token,
) -> APIResult<Json<Vec<CurrentUserGuild>>> {
    let guilds = {
        let mut conn = state.redis.get().await?;
        let data: Option<String> = conn
            .get(&format!("dashboard:user:guild:{}", token.user_id))
            .await?;
        if let Some(data) = data {
            serde_json::from_str(&data)?
        } else {
            let access_token = db::get_access_token(&state.pool, token.user_id as i64).await?;
            let http = HttpClient::new(format!("Bearer {}", access_token));
            let guilds = http.current_user_guilds().await?.model().await?;
            conn.set_ex::<_, _, ()>(
                format!("dashboard:user:guild:{}", token.user_id),
                serde_json::to_string(&guilds)?,
                60,
            )
            .await?;
            guilds
        }
    };

    Ok(Json(guilds))
}

async fn permission_checker(
    state: Arc<AppState>,
    guild_id: u64,
    user_id: u64,
) -> anyhow::Result<bool> {
    let member = state
        .http
        .guild_member(Id::new(guild_id), Id::new(user_id))
        .await?
        .model()
        .await?;
    let guild_roles = state.http.roles(Id::new(guild_id)).await?.model().await?;
    let member_roles = guild_roles
        .iter()
        .filter(|role| member.roles.contains(&role.id))
        .map(|role| (role.id, role.permissions))
        .collect::<Vec<_>>();
    let calculator = PermissionCalculator::new(
        Id::new(guild_id),
        Id::new(user_id),
        Permissions::empty(),
        &member_roles,
    );
    if calculator.root().contains(Permissions::ADMINISTRATOR) {
        return Ok(true);
    }
    Ok(false)
}

pub async fn get_guild(
    State(state): State<Arc<AppState>>,
    _token: Token,
    Path(guild_id): Path<u64>,
) -> APIResult<Json<Guild>> {
    let guild = {
        let mut conn = state.redis.get().await?;
        let data: Option<String> = conn.get(&format!("dashboard:guild:{}", guild_id)).await?;
        if let Some(data) = data {
            serde_json::from_str(&data)?
        } else {
            let result = state.http.guild(Id::new(guild_id)).await;
            let response = if let Err(_error) = result {
                return Err(APIError::notfound("Not found"));
            } else {
                result?
            };
            let guild = response.model().await?;
            conn.set_ex::<_, _, ()>(
                format!("dashboard:guild:{}", guild_id),
                serde_json::to_string(&guild)?,
                60,
            )
            .await?;
            guild
        }
    };

    Ok(Json(guild))
}

pub async fn get_guild_roles(
    State(state): State<Arc<AppState>>,
    token: Token,
    Path(guild_id): Path<u64>,
) -> APIResult<Json<Vec<Role>>> {
    if !permission_checker(Arc::clone(&state), guild_id, token.user_id).await? {
        return Err(APIError::forbitten(
            "You don't have permission to access this guild",
        ));
    }
    let roles = {
        let mut conn = state.redis.get().await?;
        let data: Option<String> = conn
            .get(&format!("dashboard:guild:{}:roles", guild_id))
            .await?;
        if let Some(data) = data {
            serde_json::from_str(&data)?
        } else {
            let roles = state.http.roles(Id::new(guild_id)).await?.model().await?;
            conn.set_ex::<_, _, ()>(
                format!("dashboard:guild:{}:roles", guild_id),
                serde_json::to_string(&roles)?,
                60,
            )
            .await?;
            roles
        }
    };
    let roles = roles
        .iter()
        .filter(|role| role.name != "@everyone")
        .cloned()
        .collect();

    Ok(Json(roles))
}

pub async fn get_guild_text_channels(
    State(state): State<Arc<AppState>>,
    token: Token,
    Path(guild_id): Path<u64>,
) -> APIResult<Json<Vec<Channel>>> {
    if !permission_checker(Arc::clone(&state), guild_id, token.user_id).await? {
        return Err(APIError::forbitten(
            "You don't have permission to access this guild",
        ));
    }
    let channels = {
        let mut conn = state.redis.get().await?;
        let data: Option<String> = conn
            .get(&format!("dashboard:guild:{}:channels", guild_id))
            .await?;
        if let Some(data) = data {
            serde_json::from_str(&data)?
        } else {
            let channels = state
                .http
                .guild_channels(Id::new(guild_id))
                .await?
                .model()
                .await?;
            conn.set_ex::<_, _, ()>(
                format!("dashboard:guild:{}:channels", guild_id),
                serde_json::to_string(&channels)?,
                60,
            )
            .await?;
            channels
        }
    };
    let channels = channels
        .iter()
        .filter(|channel| channel.kind == ChannelType::GuildText)
        .cloned()
        .collect();

    Ok(Json(channels))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildGeneralSettings {
    email_pattern: String,
    role_id: String,
    channel_id: String,
}

pub async fn set_guild_general_settings(
    State(state): State<Arc<AppState>>,
    token: Token,
    Path(guild_id): Path<u64>,
    Json(body): Json<GuildGeneralSettings>,
) -> APIResult<()> {
    let role_id = body.role_id.parse::<i64>()?;
    let channel_id = body.channel_id.parse::<u64>()?;

    if !permission_checker(Arc::clone(&state), guild_id, token.user_id).await? {
        return Err(APIError::forbitten(
            "You don't have permission to access this guild",
        ));
    }

    verify_db::add_guild(
        &state.pool,
        guild_id as i64,
        body.email_pattern.clone(),
        role_id,
        channel_id as i64,
    )
    .await?;

    let embed = EmbedBuilder::new()
        .title("認証パネル")
        .description("ボタンをクリックすると認証が始まります。")
        .build();
    let components = ComponentsBuilder::new()
        .buttons(vec![ButtonBuilder::with_custom_id(
            "auth".to_string(),
            "認証する".to_string(),
            ButtonStyle::Success,
        )
        .build()])
        .build();
    state
        .http
        .create_message(Id::new(channel_id))
        .embeds(&[embed])?
        .components(&components)?
        .await?;

    Ok(())
}

pub async fn get_guild_general_settings(
    State(state): State<Arc<AppState>>,
    token: Token,
    Path(guild_id): Path<u64>,
) -> APIResult<Json<GuildGeneralSettings>> {
    if !permission_checker(Arc::clone(&state), guild_id, token.user_id).await? {
        return Err(APIError::forbitten(
            "You don't have permission to access this guild",
        ));
    }

    let (email_pattern, role_id, channel_id) = verify_db::get_guild(&state.pool, guild_id as i64)
        .await?
        .ok_or_else(|| APIError::notfound("Not found"))?;

    Ok(Json(GuildGeneralSettings {
        email_pattern,
        role_id: role_id.to_string(),
        channel_id: channel_id.to_string(),
    }))
}

#[derive(Deserialize)]
pub struct RequestAddMailAddress {
    mail: String,
}

#[derive(Serialize)]
pub struct ResponseAddMailAddress {
    id: i64,
}

pub async fn add_mail_address(
    State(state): State<Arc<AppState>>,
    token: Token,
    Path(guild_id): Path<u64>,
    Json(body): Json<RequestAddMailAddress>,
) -> APIResult<Json<ResponseAddMailAddress>> {
    if !permission_checker(Arc::clone(&state), guild_id, token.user_id).await? {
        return Err(APIError::forbitten(
            "You don't have permission to access this guild",
        ));
    }

    let mail = body.mail.clone();
    let mail_id = mail_db::add_mail_address(&state.pool, guild_id as i64, mail).await?;

    Ok(Json(ResponseAddMailAddress { id: mail_id }))
}

#[derive(Serialize)]
pub struct ResponseGetAllMailAddress {
    mail: String,
    id: i64,
    guild_id: i64,
}

pub async fn get_all_mail_addresses(
    State(state): State<Arc<AppState>>,
    token: Token,
    Path(guild_id): Path<u64>,
) -> APIResult<Json<Vec<ResponseGetAllMailAddress>>> {
    if !permission_checker(Arc::clone(&state), guild_id, token.user_id).await? {
        return Err(APIError::forbitten(
            "You don't have permission to access this guild",
        ));
    }

    let mails = mail_db::get_all_email(&state.pool, guild_id as i64).await?;

    Ok(Json(
        mails
            .iter()
            .map(|(id, mail)| ResponseGetAllMailAddress {
                id: *id,
                mail: mail.clone(),
                guild_id: guild_id as i64,
            })
            .collect(),
    ))
}

pub async fn delete_mail_address(
    State(state): State<Arc<AppState>>,
    token: Token,
    Path((guild_id, mail_id)): Path<(u64, i64)>,
) -> APIResult<()> {
    if !permission_checker(Arc::clone(&state), guild_id, token.user_id).await? {
        return Err(APIError::forbitten(
            "You don't have permission to access this guild",
        ));
    }

    mail_db::delete_mail_address(&state.pool, guild_id as i64, mail_id).await?;

    Ok(())
}
