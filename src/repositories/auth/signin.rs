use std::sync::Arc;
//TODO: use postcard::{to_allocvec};
use redis::AsyncTypedCommands;
use serde_json::json;
use crate::db::redis::RedisPool;
use crate::errors::db_error::DbError;
use crate::models::auth::signin::{SignInSession, SIGNIN_SESSION_LIFETIME, SIGNIN_SESSION_PREFIX};

#[async_trait::async_trait]
pub trait SignInRepository {
    async fn create(&self, session_id: &str, session_data: &SignInSession) -> Result<(), DbError>;
    async fn update(&self, session_id: &str, session_data: &SignInSession) -> Result<(), DbError>;
    async fn get(&self, session_id: &str) -> Result<Option<SignInSession>, DbError>;
    async fn delete(&self, session_id: &str) -> Result<(), DbError>;
}

pub struct SignInRepositoryRedis { pub pool: Arc<RedisPool> , }

#[async_trait::async_trait]
impl SignInRepository for SignInRepositoryRedis {
    async fn create(&self, session_id: &str, session_data: &SignInSession) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds: u64 = SIGNIN_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNIN_SESSION_PREFIX, session_id);

        //TODO: optimize storage with postcard
        // let raw_data = to_allocvec(session_data);

        let _: () = conn.set_ex(key.clone(), json!(session_data).to_string(), expiration_seconds).await?;

        Ok(())
    }

    async fn update(&self, session_id: &str, session_data: &SignInSession) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds: u64 = SIGNIN_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNIN_SESSION_PREFIX, session_id);

        let _ = conn.set_ex(key.clone(), json!(session_data).to_string(), expiration_seconds).await?;

        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<Option<SignInSession>, DbError> {
        let mut conn = self.pool.get().await?;

        let expiration_seconds = SIGNIN_SESSION_LIFETIME * 60;

        let key = format!("{}:{}", SIGNIN_SESSION_PREFIX, session_id);

        let session_data_string: Option<String> = conn.get(key.clone()).await?;

        match session_data_string {
            Some(data_str) => {
                let user_data: SignInSession = serde_json::from_str(&data_str).map_err(|e| DbError::Unknown(e.to_string()))?;

                let _: bool = conn.expire(key, expiration_seconds as i64).await?;
                Ok(Some(user_data))
            },
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), DbError> {
        let mut conn = self.pool.get().await?;

        let key = format!("{}:{}", SIGNIN_SESSION_PREFIX, session_id);

        let _: usize = conn.del(key).await?;

        Ok(())
    }
}