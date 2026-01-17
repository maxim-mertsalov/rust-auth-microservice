use std::sync::Arc;
use redis::AsyncTypedCommands;
use serde_json::json;
use crate::db::redis::RedisPool;
use crate::errors::db_error::DbError;
use crate::models::auth::reset_auth::{ResetAuth, RESET_AUTH_LIFETIME, RESET_AUTH_PREFIX};

#[async_trait::async_trait]
pub trait ResetAuthRepository {
    /// Create a new password reset session for a user, returning a `token`
    async fn create(&self, user_id: &str) -> Result<String, DbError>;
    /// Get a password reset session by token
    async fn get(&self, token: &str) -> Result<Option<ResetAuth>, DbError>;
    /// Delete a password reset session by `token`
    async fn delete(&self, token: &str) -> Result<(), DbError>;
}

pub struct ResetAuthRepositoryRedis { pub pool: Arc<RedisPool> , }

#[async_trait::async_trait]
impl ResetAuthRepository for ResetAuthRepositoryRedis {
    async fn create(&self, user_id: &str) -> Result<String, DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = RESET_AUTH_LIFETIME * 60;

        let token = uuid::Uuid::new_v4().to_string();
        let key = format!("{}:{}", RESET_AUTH_PREFIX, token.clone());

        let reset_auth = ResetAuth {
            user_id: user_id.to_string(),
        };

        let _: () = conn.set(key.clone(), json!(reset_auth).to_string()).await?;

        let _: bool = conn.expire(key, expiration_seconds).await?;

        Ok(token)
    }

    async fn get(&self, token: &str) -> Result<Option<ResetAuth>, DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", RESET_AUTH_PREFIX, token);

        let reset_auth_string: Option<String> = conn.get(key).await?;

        match reset_auth_string {
            Some(data_str) => {
                let reset_auth: ResetAuth = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;
                Ok(Some(reset_auth))
            },
            None => Ok(None),
        }
    }

    async fn delete(&self, token: &str) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", RESET_AUTH_PREFIX, token);

        let _: usize = conn.del(key).await?;

        Ok(())
    }
}
