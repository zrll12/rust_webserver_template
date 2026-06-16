use std::future::Future;
use std::pin::Pin;
use axum::Router;
use ws_core::{error::AppError, module::AppModule, state::AppState};

pub mod entity;
pub mod error;
pub mod extract;
pub mod routes;
pub mod service;

#[allow(unused_imports)]
pub use service::{TokenInfo, UserService};

pub struct UserModule;

impl AppModule for UserModule {
    fn name(&self) -> &'static str {
        "user"
    }

    fn routes(&self) -> Router<AppState> {
        routes::router()
    }

    fn init<'a>(&'a self, state: &'a AppState) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            state.set_module(UserService::new(state.db.clone()));
            Ok(())
        })
    }
}
