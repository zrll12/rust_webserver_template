use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use ws_core::{error::AppError, extract::ModuleExt, state::AppState};
use super::service::{TokenInfo, UserService};

/// 从请求头 `Authorization: Bearer <token>` 中提取并验证 token，返回用户信息。
/// 使用方式：`TokenInfo(user): TokenInfo`
impl FromRequestParts<AppState> for TokenInfo {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::InvalidToken)?
            .to_owned();

        let ModuleExt(svc) = ModuleExt::<UserService>::from_request_parts(parts, state).await?;
        svc.verify_token(&token).await.map_err(AppError::from)
    }
}
