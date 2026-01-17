use std::sync::Arc;
use redis::AsyncTypedCommands;
use serde_json::json;
use crate::db::redis::RedisPool;
use crate::errors::db_error::DbError;
use crate::models::auth::signup::{SignUpData, SignUpSession, SIGNUP_SESSION_LIFETIME, SIGNUP_SESSION_PREFIX};

#[async_trait::async_trait]
pub trait SignUpRepository {
    async fn create(&self, session_id: &str, session_data: &SignUpSession) -> Result<(), DbError>;
    async fn update(&self, session_id: &str, session_data: &SignUpSession) -> Result<(), DbError>;
    async fn update_data(&self, session_id: &str, user_data: &SignUpData) -> Result<(), DbError>;
    async fn update_code(&self, session_id: &str, code: &str) -> Result<(), DbError>;
    async fn increment_attempts(&self, session_id: &str) -> Result<u8, DbError>;
    async fn get(&self, session_id: &str) -> Result<Option<SignUpSession>, DbError>;
    async fn delete(&self, session_id: &str) -> Result<(), DbError>;
}

pub struct SignUpRepositoryRedis { pub pool: Arc<RedisPool> , }

#[async_trait::async_trait]
impl SignUpRepository for SignUpRepositoryRedis {
    async fn create(&self, session_id: &str, session_data: &SignUpSession) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = SIGNUP_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNUP_SESSION_PREFIX, session_id);

        let _: () = conn.set(key.clone(), json!(session_data).to_string()).await?;

        let _: bool = conn.expire(key, expiration_seconds).await?;

        Ok(())
    }

    async fn update(&self, session_id: &str, session_data: &SignUpSession) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = SIGNUP_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNUP_SESSION_PREFIX, session_id);

        let _ = conn.set(key.clone(), json!(session_data).to_string()).await?;

        let _: bool = conn.expire(key.clone(), expiration_seconds).await?;

        Ok(())
    }

    async fn update_data(&self, session_id: &str, user_data: &SignUpData) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = SIGNUP_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNUP_SESSION_PREFIX, session_id);

        let session_data_string: Option<String> = conn.get(key.clone()).await?;

        match session_data_string {
            Some(data_str) => {
                let mut session_data: SignUpSession = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;
                session_data.data = user_data.clone();
                let _: () = conn.set(key.clone(), json!(session_data).to_string()).await?;

                let _: bool = conn.expire(key, expiration_seconds).await?;
                Ok(())
            },
            None => Err(DbError::NotFound("Signup session is not found".to_string())),
        }
    }

    async fn update_code(&self, session_id: &str, code: &str) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = SIGNUP_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNUP_SESSION_PREFIX, session_id);

        let session_data_string: Option<String> = conn.get(key.clone()).await?;

        match session_data_string {
            Some(data_str) => {
                let mut session_data: SignUpSession = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;
                session_data.verification_code = code.to_string();
                session_data.attempts = 0;
                let _: () = conn.set(key.clone(), json!(session_data).to_string()).await?;

                let _: bool = conn.expire(key, expiration_seconds).await?;
                Ok(())
            },
            None => Err(DbError::NotFound("Signup session is not found".to_string())),
        }
    }

    async fn increment_attempts(&self, session_id: &str) -> Result<u8, DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = SIGNUP_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNUP_SESSION_PREFIX, session_id);

        let session_data_string: Option<String> = conn.get(key.clone()).await?;

        match session_data_string {
            Some(data_str) => {
                let mut session_data: SignUpSession = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;
                session_data.attempts += 1;
                let _: () = conn.set(key.clone(), json!(session_data).to_string()).await?;

                let _: bool = conn.expire(key, expiration_seconds).await?;
                Ok(session_data.attempts)
            },
            None => Err(DbError::NotFound("Signup session is not found".to_string())),
        }
    }

    async fn get(&self, session_id: &str) -> Result<Option<SignUpSession>, DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = SIGNUP_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNUP_SESSION_PREFIX, session_id);

        let session_data_string: Option<String> = conn.get(key.clone()).await?;

        match session_data_string {
            Some(data_str) => {
                let user_data: SignUpSession = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;

                let _: bool = conn.expire(key, expiration_seconds).await?;
                Ok(Some(user_data))
            },
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", SIGNUP_SESSION_PREFIX, session_id);

        let _: usize = conn.del(key).await?;

        Ok(())
    }
}