use axum::Router;
use crate::error::AppError;
use crate::state::AppState;

pub trait AppModule: Send + Sync + 'static {
    /// Module identifier used in logs and diagnostics.
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

/// Pairs a module with its HTTP route prefix.
///
/// # Examples
/// ```
/// // prefix defaults to module name
/// let e: ModuleEntry = PingModule.into();
///
/// // custom prefix
/// let e: ModuleEntry = (PingModule, "info").into();
/// ```
pub struct ModuleEntry {
    pub module: Box<dyn AppModule>,
    pub prefix: &'static str,
}

impl<M: AppModule> From<M> for ModuleEntry {
    fn from(m: M) -> Self {
        let prefix = m.name();
        ModuleEntry { module: Box::new(m), prefix }
    }
}

impl<M: AppModule> From<(M, &'static str)> for ModuleEntry {
    fn from((m, prefix): (M, &'static str)) -> Self {
        ModuleEntry { module: Box::new(m), prefix }
    }
}
