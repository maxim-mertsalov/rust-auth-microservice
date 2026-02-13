use std::sync::Arc;
use redis::AsyncTypedCommands;
use serde_json::json;
use crate::db::redis::RedisPool;
use crate::errors::db_error::DbError;
use crate::models::auth::email_verification::{EmailVerification, EMAIL_VERIFICATION_LIFETIME, EMAIL_VERIFICATION_PREFIX};

#[async_trait::async_trait]
pub trait EmailVerificationRepository {
    async fn create(&self, data: &EmailVerification) -> Result<String, DbError>;
    async fn get(&self, token: &str) -> Result<Option<EmailVerification>, DbError>;
    async fn update(&self, token: &str, data: &EmailVerification) -> Result<(), DbError>;
    async fn delete(&self, token: &str) -> Result<(), DbError>;
}

pub struct EmailVerificationRepositoryRedis { pub pool: Arc<RedisPool> , }

#[async_trait::async_trait]
impl EmailVerificationRepository for EmailVerificationRepositoryRedis {
    async fn create(&self, data: &EmailVerification) -> Result<String, DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = EMAIL_VERIFICATION_LIFETIME * 60;

        let token = uuid::Uuid::new_v4().to_string();
        let key = format!("{}:{}", EMAIL_VERIFICATION_PREFIX, token.clone());

        let _: () = conn.set_ex(key.clone(), json!(data).to_string(), expiration_seconds).await?;

        Ok(token)
    }

    async fn get(&self, token: &str) -> Result<Option<EmailVerification>, DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", EMAIL_VERIFICATION_PREFIX, token);

        let reset_auth_string: Option<String> = conn.get(key).await?;

        match reset_auth_string {
            Some(data_str) => {
                let reset_auth: EmailVerification = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;
                Ok(Some(reset_auth))
            },
            None => Ok(None),
        }
    }

    async fn update(&self, token: &str, data: &EmailVerification) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", EMAIL_VERIFICATION_PREFIX, token);

        let _ = conn.set(key.clone(), json!(data).to_string()).await?;

        Ok(())
    }

    async fn delete(&self, token: &str) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", EMAIL_VERIFICATION_PREFIX, token);

        let _: usize = conn.del(key).await?;

        Ok(())
    }
}
