use axum::{Json, Router, routing::{get, post}};
use serde::{Deserialize, Serialize};
use thalos_core::{error::AppError, extract::ModuleExt, state::AppState};
use super::{error::UserError, service::{TokenInfo, UserService}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/verify", post(verify))
        .route("/me", get(me))
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Deserialize)]
pub(super) struct RegisterRequest {
    username: String,
    password: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Serialize)]
pub(super) struct RegisterResponse {
    id: i32,
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/user/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered", body = RegisterResponse),
        (status = 400, description = "Invalid field"),
    )
))]
pub(super) async fn register(
    ModuleExt(svc): ModuleExt<UserService>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let id = svc.register(&body.username, &body.password).await.map_err(UserError::from)?;
    Ok(Json(RegisterResponse { id }))
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Deserialize)]
pub(super) struct VerifyRequest {
    username: String,
    password: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Serialize)]
pub(super) struct VerifyResponse {
    id: i32,
    token: String,
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/user/verify",
    request_body = VerifyRequest,
    responses(
        (status = 200, description = "Login successful", body = VerifyResponse),
        (status = 401, description = "Invalid credentials"),
    )
))]
pub(super) async fn verify(
    ModuleExt(svc): ModuleExt<UserService>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    let id = svc.verify(&body.username, &body.password).await.map_err(UserError::from)?;
    let token = svc.issue_token(id);
    Ok(Json(VerifyResponse { id, token }))
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Serialize)]
pub(super) struct MeResponse {
    user_id: i32,
    username: String,
}

/// 需要 token 的接口示例：GET /user/me，Header: Authorization: Bearer <token>
#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/user/me",
    responses(
        (status = 200, description = "Current user info", body = MeResponse),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("bearer_token" = []))
))]
pub(super) async fn me(TokenInfo { user_id, username }: TokenInfo) -> Result<Json<MeResponse>, AppError> {
    Ok(Json(MeResponse { user_id, username }))
}
