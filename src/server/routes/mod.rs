pub mod auth;
pub mod dashboard;

use crate::AppState;
use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct GetInviteUrlResponse {
    pub url: String,
}

pub async fn invite_url(State(state): State<Arc<AppState>>) -> Json<GetInviteUrlResponse> {
    let invite_url = format!("https://discord.com/oauth2/authorize?client_id={}&permissions=8&integration_type=0&scope=bot", state.application_id);
    Json(GetInviteUrlResponse { url: invite_url })
}
