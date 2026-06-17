use std::sync::Arc;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::error::AppError;
use crate::state::AppState;

/// 从 AppState 的 module extension map 中提取服务。
/// 使用方式：`ModuleExt(svc): ModuleExt<MyService>`
pub struct ModuleExt<T>(pub Arc<T>);

impl<T> FromRequestParts<AppState> for ModuleExt<T>
where
    T: Send + Sync + 'static,
{
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        state
            .get_module::<T>()
            .map(ModuleExt)
            .ok_or(AppError::NotFound)
    }
}
