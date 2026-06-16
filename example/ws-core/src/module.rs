use std::future::Future;
use std::pin::Pin;
use axum::Router;
use crate::error::AppError;
use crate::state::AppState;

pub trait AppModule: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn routes(&self) -> Router<AppState>;

    /// 模块初始化，按 all_modules() 声明顺序依次执行。
    /// 需要向其他模块暴露服务的模块在此调用 state.set_module(...)。
    fn init<'a>(&'a self, _state: &'a AppState) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
}
