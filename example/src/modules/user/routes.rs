use axum::{Json, Router, routing::{get, post}};
use serde::{Deserialize, Serialize};
use ws_core::{error::AppError, extract::ModuleExt, state::AppState};
use super::{error::UserError, service::{TokenInfo, UserService}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/verify", post(verify))
        .route("/me", get(me))
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    id: i32,
}

async fn register(
    ModuleExt(svc): ModuleExt<UserService>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let id = svc.register(&body.username, &body.password).await.map_err(UserError::from)?;
    Ok(Json(RegisterResponse { id }))
}

#[derive(Deserialize)]
struct VerifyRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    id: i32,
    token: String,
}

async fn verify(
    ModuleExt(svc): ModuleExt<UserService>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    let id = svc.verify(&body.username, &body.password).await.map_err(UserError::from)?;
    let token = svc.issue_token(id);
    Ok(Json(VerifyResponse { id, token }))
}

#[derive(Serialize)]
struct MeResponse {
    user_id: i32,
    username: String,
}

/// 需要 token 的接口示例：GET /user/me，Header: Authorization: Bearer <token>
async fn me(TokenInfo { user_id, username }: TokenInfo) -> Result<Json<MeResponse>, AppError> {
    Ok(Json(MeResponse { user_id, username }))
}
