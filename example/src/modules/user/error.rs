use thiserror::Error;
use ws_core::error::AppError;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Username already taken")]
    UsernameTaken,
    #[error("User not found")]
    NotFound,
    #[error("Wrong password")]
    WrongPassword,
    #[error("Invalid token")]
    InvalidToken,
}

impl From<UserError> for AppError {
    fn from(e: UserError) -> Self {
        match e {
            UserError::NotFound => AppError::NotFound,
            UserError::UsernameTaken => AppError::InvalidField {
                field: "username".into(),
                reason: e.to_string(),
            },
            UserError::WrongPassword | UserError::InvalidToken => AppError::InvalidToken,
        }
    }
}
