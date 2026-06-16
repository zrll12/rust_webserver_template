use thiserror::Error;
use ws_core::error::AppError;

#[derive(Debug, Error)]
pub enum CommentError {
    #[error("Comment not found")]
    NotFound,
    #[error("Permission denied")]
    NotOwner,
}

impl From<CommentError> for AppError {
    fn from(e: CommentError) -> Self {
        match e {
            CommentError::NotFound => AppError::NotFound,
            CommentError::NotOwner => AppError::PermissionDenied,
        }
    }
}
