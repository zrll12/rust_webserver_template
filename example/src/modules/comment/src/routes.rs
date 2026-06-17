use axum::{Json, Router, extract::State, routing::{get, post}};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use sea_orm::ActiveValue::NotSet;
use serde::{Deserialize, Serialize};
use thalos_core::{error::AppError, state::AppState};
use crate::entity::{ActiveModel, Entity};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/", post(create))
}

#[derive(Serialize)]
struct CommentDto {
    id: i32,
    author_id: i32,
    content: String,
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<CommentDto>>, AppError> {
    let comments = Entity::find()
        .all(&state.db)
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(Json(
        comments
            .into_iter()
            .map(|c| CommentDto {
                id: c.id,
                author_id: c.author_id,
                content: c.content,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
struct CreateRequest {
    author_id: i32,
    content: String,
}

async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateRequest>,
) -> Result<Json<CommentDto>, AppError> {
    let comment = ActiveModel {
        id: NotSet,
        author_id: Set(body.author_id),
        content: Set(body.content),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::DatabaseError)?;

    Ok(Json(CommentDto {
        id: comment.id,
        author_id: comment.author_id,
        content: comment.content,
    }))
}
