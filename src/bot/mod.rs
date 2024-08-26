use crate::utils::state::AppState;

use std::env;
use std::sync::Arc;

use once_cell::sync::Lazy;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle, Component};
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use url::Url;
use uuid::Uuid;

static BASE_AUTH_URL: Lazy<String> =
    Lazy::new(|| format!("{}/auth", env::var("BASE_URL").unwrap()));

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
                url.query_pairs_mut().append_pair("code", &code.to_string());
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

pub async fn receive_event(state: Arc<AppState>, event: Event) -> anyhow::Result<()> {
    match event {
        Event::InteractionCreate(interaction) => {
            create_interaction(state, interaction.0).await?;
        }
        _ => {}
    }
    Ok(())
}

pub async fn run_bot(state: Arc<AppState>, token: String) -> anyhow::Result<()> {
    let mut shard = Shard::new(ShardId::ONE, token, Intents::GUILDS);

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

        tracing::debug!("Received event: {:?}", event);

        tokio::spawn(receive_event(Arc::clone(&state), event));
    }

    Ok(())
}
