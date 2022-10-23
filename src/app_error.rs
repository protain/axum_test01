use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Response, Json};
use serde_json::json;
use std::borrow::Cow;

pub enum AppError {
    InternalServerError(anyhow::Error),
    AnyStatusError(StatusCode, anyhow::Error),
    ValidationError,
}

impl AppError {
    pub fn server_err<T: std::fmt::Debug>(err: &T) -> Self {
        AppError::InternalServerError(anyhow::anyhow!(format!("{:?}", err)))
    }
    pub fn make_err<T: std::fmt::Debug>(code: u16, err: &T) -> Self {
        AppError::AnyStatusError(
            StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            anyhow::anyhow!(format!("{:?}", err)),
        )
    }
}

impl From<anyhow::Error> for AppError {
    fn from(inner: anyhow::Error) -> Self {
        AppError::InternalServerError(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(inner) => {
                tracing::debug!("stacktrace: {}", inner.backtrace());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Cow::Owned(format!("{:?}", inner)),
                )
            }
            AppError::AnyStatusError(stat, inner) => {
                tracing::debug!("stacktrace: {}", inner.backtrace());
                (stat.clone(), Cow::Owned(format!("{:?}", inner)))
            }
            AppError::ValidationError => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Cow::Borrowed("validation errors"),
            ),
        };
        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
