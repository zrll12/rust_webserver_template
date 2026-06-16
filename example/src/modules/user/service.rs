use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sea_orm::ActiveValue::NotSet;
use crate::modules::user::entity::user::{ActiveModel, Column, Entity};
use super::error::UserError;

pub struct TokenInfo {
    pub user_id: i32,
    pub username: String,
}

#[derive(Clone)]
pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<i32, UserError> {
        let exists = Entity::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await
            .expect("db error");
        if exists.is_some() {
            return Err(UserError::UsernameTaken);
        }

        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .expect("bcrypt failed");

        let user = ActiveModel {
            id: NotSet,
            username: Set(username.to_owned()),
            password_hash: Set(password_hash),
            created_at: Set(chrono::Utc::now().into()),
        }
        .insert(&self.db)
        .await
        .expect("db error");

        Ok(user.id)
    }

    pub async fn verify(&self, username: &str, password: &str) -> Result<i32, UserError> {
        let user = Entity::find()
            .filter(Column::Username.eq(username))
            .one(&self.db)
            .await
            .expect("db error")
            .ok_or(UserError::NotFound)?;

        if bcrypt::verify(password, &user.password_hash).unwrap_or(false) {
            Ok(user.id)
        } else {
            Err(UserError::WrongPassword)
        }
    }

    /// 签发 token：使用 user_id 的 base64 编码作为示例，实际项目替换为 JWT。
    pub fn issue_token(&self, user_id: i32) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "uid:{}", user_id).unwrap();
        // 生产环境：用 jsonwebtoken crate 签名
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, s.as_bytes())
    }

    /// 从 token 中还原用户信息，取不到则返回 InvalidToken。
    pub async fn verify_token(&self, token: &str) -> Result<TokenInfo, UserError> {
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            token,
        )
        .map_err(|_| UserError::InvalidToken)?;

        let s = String::from_utf8(decoded).map_err(|_| UserError::InvalidToken)?;
        let user_id: i32 = s
            .strip_prefix("uid:")
            .and_then(|v| v.parse().ok())
            .ok_or(UserError::InvalidToken)?;

        let user = Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .expect("db error")
            .ok_or(UserError::InvalidToken)?;

        Ok(TokenInfo {
            user_id: user.id,
            username: user.username,
        })
    }
}
