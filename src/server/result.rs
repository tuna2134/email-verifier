use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

pub type AppResult<T> = Result<T, AppError>;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseAPIError {
    pub status: u16,
    pub message: String,
}

pub type APIResult<T> = Result<T, APIError>;

pub struct APIError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let response = Json(ResponseAPIError {
            status: self.status.as_u16(),
            message: self.message,
        });
        (self.status, response).into_response()
    }
}

impl<E> From<E> for APIError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.into().to_string(),
        }
    }
}

impl APIError {
    pub fn notfound(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    pub fn forbitten(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.to_string(),
        }
    }

    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.to_string(),
        }
    }
}
