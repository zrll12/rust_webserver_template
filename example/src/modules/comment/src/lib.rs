use axum::Router;
use thalos_core::{module::AppModule, state::AppState};

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


}
