use axum::Router;
use crate::modules::core::AppError;
use crate::state::AppState;

pub trait AppModule: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn routes(&self) -> Router<AppState>;

    fn init(&self, _state: &AppState) -> Result<(), AppError> {
        Ok(())
    }

    fn shutdown(&self) {}

    #[cfg(feature = "openapi")]
    fn openapi(&self) -> utoipa::openapi::OpenApi {
        utoipa::openapi::OpenApiBuilder::new().build()
    }
}

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
