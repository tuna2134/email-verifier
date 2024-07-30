use crate::db::verify as db;
use crate::utils::state::AppState;

use std::sync::Arc;
use std::env;

use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle, Component};
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::{marker::RoleMarker, Id};
use twilight_util::builder::embed::EmbedBuilder;
use uuid::Uuid;
use vesper::prelude::*;
use once_cell::sync::Lazy;
use url::Url;

#[command]
#[description = "認証"]
#[required_permissions(MANAGE_ROLES)]
async fn auth(
    ctx: &mut SlashContext<Arc<AppState>>,
    #[description = "メールアドレスのパターン"] email_pattern: String,
    #[description = "付与するロール"] role: Id<RoleMarker>,
) -> DefaultCommandResult {
    db::add_guild(
        &ctx.data.pool,
        ctx.interaction.guild_id.unwrap().get() as i64,
        email_pattern,
        role.get() as i64,
    )
    .await?;
    let embed = EmbedBuilder::new()
        .title("認証パネル")
        .description("ボタンをクリックすると認証が始まります。")
        .build();
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    embeds: Some(vec![embed]),
                    components: Some(vec![Component::ActionRow(ActionRow {
                        components: vec![Component::Button(Button {
                            style: ButtonStyle::Success,
                            label: Some("認証を開始する".to_string()),
                            custom_id: Some("auth".to_string()),
                            url: None,
                            emoji: None,
                            disabled: false,
                        })],
                    })]),
                    ..Default::default()
                }),
            },
        )
        .await?;
    Ok(())
}

static BASE_AUTH_URL: Lazy<String> = Lazy::new(|| env::var("AUTH_PAGE_URL").unwrap());

async fn create_interaction(state: Arc<AppState>, interaction: Interaction) -> anyhow::Result<()> {
    if interaction.kind == InteractionType::MessageComponent {
        if let Some(InteractionData::MessageComponent(data)) = &interaction.data {
            if data.custom_id.as_str() == "auth" {
                let code = Uuid::new_v4();
                {
                    let mut cache = state.cache.lock().await;
                    cache.insert(
                        format!("auth:{}", code),
                        format!(
                            "{}:{}",
                            interaction.member.clone().unwrap().user.unwrap().id,
                            interaction.guild_id.unwrap()
                        ),
                    );
                };
                let mut url = Url::parse(BASE_AUTH_URL.as_str())?;
                url.query_pairs_mut()
                    .append_pair("code", &code.to_string());
                state.interaction()
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(InteractionResponseData {
                                content: Some("認証を開始します。\n以下のボタンをクリックして飛んでください。".to_string()),
                                flags: Some(MessageFlags::EPHEMERAL),
                                components: Some(vec![
                                    Component::ActionRow(ActionRow {
                                        components: vec![Component::Button(Button {
                                            style: ButtonStyle::Link,
                                            label: Some("認証ページへ".to_string()),
                                            custom_id: None,
                                            url: Some(url.to_string()),
                                            emoji: None,
                                            disabled: false,
                                        })],
                                    }),
                                ]),
                                ..Default::default()
                            }),
                        },
                    )
                    .await?;
            }
        }
    }
    Ok(())
}

pub async fn run_bot(state: Arc<AppState>, token: String) -> anyhow::Result<()> {
    let mut shard = Shard::new(ShardId::ONE, token, Intents::GUILDS);

    let framework = Arc::new(
        Framework::builder(state.http.clone(), state.application_id, Arc::clone(&state))
            .command(auth)
            .build(),
    );

    framework.register_global_commands().await?;

    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(error) => {
                tracing::warn!("Error receiving event: {:?}", error);
                if error.is_fatal() {
                    break;
                }
                continue;
            }
        };
        if let Event::InteractionCreate(interaction) = event {
            let clone = Arc::clone(&framework);
            let state_clone = Arc::clone(&state);
            tokio::spawn(async move {
                let inner = &interaction.0;
                clone.process(inner.clone()).await;
                create_interaction(state_clone, inner.clone())
                    .await
                    .unwrap();
            });
        }
    }

    Ok(())
}
