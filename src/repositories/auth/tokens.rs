use std::sync::Arc;
use redis::{AsyncTypedCommands};
use serde_json::json;
use crate::db::redis::RedisPool;
use crate::models::auth::tokens::{TokenSession, TOKEN_SESSION_PREFIX};
use crate::errors::db_error::DbError;

#[async_trait::async_trait]
pub trait TokenSessionRepository {
    async fn create(&self, refresh_token: &str, tokes_session: &TokenSession) -> Result<(), DbError>;
    async fn delete(&self, refresh_token: &str) -> Result<(), DbError>;
    async fn get(&self, refresh_token: &str) -> Result<Option<TokenSession>, DbError>;
    async fn set_expiration(&self, refresh_token: &str, expire_in_days: i64) -> Result<(), DbError>;
    async fn rename(&self, old_refresh_token: &str, new_refresh_token: &str, expire_in_days: i64) -> Result<(), DbError>;
}

pub struct TokenSessionRepositoryRedis { pub pool: Arc<RedisPool> }

#[async_trait::async_trait]
impl TokenSessionRepository for TokenSessionRepositoryRedis {
    async fn create(&self, refresh_token: &str, tokes_session: &TokenSession) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", TOKEN_SESSION_PREFIX, refresh_token);

        let expiration_seconds: u64 = (tokes_session.expires_at.timestamp() - chrono::Utc::now().timestamp()) as u64;

        let _: () = conn.set_ex(key.clone(), json!(tokes_session).to_string(), expiration_seconds).await?;

        Ok(())
    }

    async fn delete(&self, refresh_token: &str) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", TOKEN_SESSION_PREFIX, refresh_token);

        let _: usize = conn.del(key).await?;

        Ok(())
    }

    async fn get(&self, refresh_token: &str) -> Result<Option<TokenSession>, DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", TOKEN_SESSION_PREFIX, refresh_token);

        let token_session_string: Option<String> = conn.get(key).await?;

        match token_session_string {
            Some(session_str) => {
                let token_session: TokenSession = serde_json::from_str(&session_str).map_err(|e| DbError::Unknown(e.to_string()))?;
                Ok(Some(token_session))
            },
            None => Ok(None),
        }
    }

    async fn set_expiration(&self, refresh_token: &str, expire_in_days: i64) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", TOKEN_SESSION_PREFIX, refresh_token);

        let expire_in = expire_in_days * 24 * 60 * 60; // convert days to seconds

        let _ = conn.expire(key, expire_in).await?;
        Ok(())
    }

    async fn rename(&self, old_refresh_token: &str, new_refresh_token: &str, expire_in_days: i64) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let old_key = format!("{}:{}", TOKEN_SESSION_PREFIX, old_refresh_token);
        let new_key = format!("{}:{}", TOKEN_SESSION_PREFIX, new_refresh_token);

        let _ = conn.rename(old_key, &new_key).await?;

        let expire_in = expire_in_days * 24 * 60 * 60; // convert days to seconds

        let _ = conn.expire(new_key, expire_in).await?;

        Ok(())
    }
}