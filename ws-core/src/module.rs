use axum::Router;
use crate::error::AppError;
use crate::state::AppState;

pub trait AppModule: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn routes(&self) -> Router<AppState>;

    /// 模块初始化，按 all_modules() 声明顺序依次执行。
    /// 需要向其他模块暴露服务的模块在此调用 state.set_module(...)。
    /// 需要异步操作时用 futures::executor::block_on(...) 包裹。
    fn init(&self, _state: &AppState) -> Result<(), AppError> {
        Ok(())
    }

    /// 返回本模块的 OpenAPI 描述（paths + schemas）。
    /// 仅在 `openapi` feature 启用时生效，框架会在启动时聚合所有模块的文档。
    /// 默认返回空文档，模块按需 override 即可。
    #[cfg(feature = "openapi")]
    fn openapi(&self) -> utoipa::openapi::OpenApi {
        utoipa::openapi::OpenApiBuilder::new().build()
    }
}
