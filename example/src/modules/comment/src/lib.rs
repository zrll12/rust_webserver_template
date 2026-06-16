use std::future::Future;
use std::pin::Pin;
use axum::Router;
use ws_core::{error::AppError, module::AppModule, state::AppState};

pub mod entity;
pub mod error;
pub mod routes;

pub struct CommentModule;

impl AppModule for CommentModule {
    fn name(&self) -> &'static str {
        "comment"
    }

    fn routes(&self) -> Router<AppState> {
        routes::router()
    }

    fn init<'a>(
        &'a self,
        _state: &'a AppState,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
}
