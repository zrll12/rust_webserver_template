use axum::Router;
use crate::error::AppError;
use crate::state::AppState;

pub trait AppModule: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn routes(&self) -> Router<AppState>;

    /// Called in `all_modules()` order. Use `state.set_module(...)` to expose services.
    /// Wrap async ops with `futures::executor::block_on`.
    fn init(&self, _state: &AppState) -> Result<(), AppError> {
        Ok(())
    }

    /// Called on shutdown in reverse `all_modules()` order.
    fn shutdown(&self) {}

    /// Returns this module's OpenAPI spec (paths + schemas). Override as needed.
    #[cfg(feature = "openapi")]
    fn openapi(&self) -> utoipa::openapi::OpenApi {
        utoipa::openapi::OpenApiBuilder::new().build()
    }
}
