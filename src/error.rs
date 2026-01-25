use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Too many submit")]
    TooManySubmit,
    #[error("Not found")]
    NotFound,
    #[error("Field {field}, {reason}")]
    InvalidField { field: String, reason: String },
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
}

impl AppError {
    fn get_status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,
            AppError::PermissionDenied => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::TooManySubmit => StatusCode::TOO_MANY_REQUESTS,
            AppError::InvalidField { .. } => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.get_status_code(), self.to_string()).into_response()
    }
}
