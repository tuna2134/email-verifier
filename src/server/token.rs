use super::result::AppError;
use crate::db::token as db;
use crate::utils::state::AppState;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use base64::prelude::*;
use getrandom::getrandom;

pub struct Token {
    pub user_id: u64,
    pub nonce: [u8; 32],
}

impl Token {
    pub fn new(user_id: u64) -> anyhow::Result<Self> {
        let mut nonce = [0; 32];
        getrandom(&mut nonce)?;
        Ok(Self { user_id, nonce })
    }

    pub fn generate(&self) -> anyhow::Result<String> {
        let mut buffer = [0; 41];
        buffer[..8].copy_from_slice(&self.user_id.to_be_bytes());
        buffer[8] = b'.';
        buffer[9..].copy_from_slice(&self.nonce);
        Ok(BASE64_URL_SAFE_NO_PAD.encode(&buffer))
    }

    pub fn parse(token: String) -> anyhow::Result<Self> {
        let buffer = BASE64_URL_SAFE_NO_PAD.decode(token.as_bytes())?;
        let mut user_id_bytes = [0u8; 8];
        user_id_bytes.copy_from_slice(&buffer[..8]);
        let user_id = u64::from_be_bytes(user_id_bytes);
        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&buffer[9..]);
        Ok(Self { user_id, nonce })
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Token {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| anyhow::anyhow!("Missing authorization header"))?;
        let token = Token::parse(bearer.token().to_string())?;

        let nonce = BASE64_URL_SAFE_NO_PAD.encode(&token.nonce);

        if db::exist_token(&state.pool, token.user_id as i64, nonce).await? {
            return Err(anyhow::anyhow!("Invalid token").into());
        }

        Ok(token)
    }
}
