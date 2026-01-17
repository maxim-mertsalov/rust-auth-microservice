use std::sync::Arc;
use redis::AsyncTypedCommands;
use serde_json::json;
use crate::db::redis::RedisPool;
use crate::errors::db_error::DbError;
use crate::models::auth::password_reset::{PasswordReset, PASSWORD_RESET_LIFETIME, PASSWORD_RESET_PREFIX};

#[async_trait::async_trait]
pub trait PasswordResetRepository {
    /// Create a new password reset session for a user, returning a `token`
    async fn create(&self, user_id: &str, otp_code: &str) -> Result<String, DbError>;
    /// Get a password reset session by token
    async fn get(&self, token: &str) -> Result<Option<PasswordReset>, DbError>;
    /// Delete a password reset session by `token`
    async fn delete(&self, token: &str) -> Result<(), DbError>;
    /// Increment the number of attempts for a password reset session and returns current attempts
    async fn increment_attempts(&self, token: &str) -> Result<u8, DbError>;
}

pub struct PasswordResetRepositoryRedis { pub pool: Arc<RedisPool> , }

#[async_trait::async_trait]
impl PasswordResetRepository for PasswordResetRepositoryRedis {
    async fn create(&self, user_id: &str, otp_code: &str) -> Result<String, DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = PASSWORD_RESET_LIFETIME * 60;

        let token = uuid::Uuid::new_v4().to_string();
        let key = format!("{}:{}", PASSWORD_RESET_PREFIX, token);

        let now = chrono::Utc::now();

        // let password_reset = PasswordReset {
        //
        // };

        // let _: () = conn.set(key.clone(), json!(password_reset).to_string()).await?;

        let _: bool = conn.expire(key, expiration_seconds).await?;

        Ok(token)
    }

    async fn get(&self, token: &str) -> Result<Option<PasswordReset>, DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", PASSWORD_RESET_PREFIX, token);

        let password_reset_string: Option<String> = conn.get(key).await?;

        match password_reset_string {
            Some(data_str) => {
                let password_reset: PasswordReset = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;
                Ok(Some(password_reset))
            },
            None => Ok(None),
        }
    }

    async fn delete(&self, token: &str) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", PASSWORD_RESET_PREFIX, token);

        let deleted: usize = conn.del(key).await?;
        if deleted == 0 {
            return Err(DbError::NotFound("Password reset session not found".to_string()));
        }

        Ok(())
    }

    async fn increment_attempts(&self, token: &str) -> Result<u8, DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", PASSWORD_RESET_PREFIX, token);

        let password_reset_string = conn.get(key.clone()).await?
            .ok_or_else(|| DbError::NotFound("Password reset session not found".to_string()))?;

        let mut password_reset: PasswordReset = serde_json::from_str(&password_reset_string).map_err(|e| DbError::Unknown(e.to_string()))?;
        password_reset.attempts += 1;

        let _: () = conn.set(key.clone(), json!(password_reset).to_string()).await?;

        Ok(password_reset.attempts)
    }
}
