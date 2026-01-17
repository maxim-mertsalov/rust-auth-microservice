use std::sync::Arc;
use crate::db::postgres::PgPool;
use crate::errors::db_error::DbError;
use crate::models::auth::sessions::{MinFieldsSession, Session, SessionStatus};

#[async_trait::async_trait]
pub trait SessionRepository {
    async fn create(&self, session: &Session) -> Result<Session, DbError>;
    async fn get_by_session_id(&self, session_id: &str) -> Result<Option<Session>, DbError>;
    async fn update_with_refresh(&self, session_id: &str, refresh_token: &str) -> Result<Session, DbError>;
    async fn terminate_with_status(&self, session_id: &str, session_status: SessionStatus) -> Result<Session, DbError>;
    async fn terminate_others_with_status(&self, user_id: &str, session_id: &str, session_status: SessionStatus) -> Result<Vec<Session>, DbError>;
    async fn get_all_by_user_full(&self, user_id: &str) -> Result<Vec<Session>, DbError>;
    async fn get_all_by_user_min(&self, user_id: &str) -> Result<Vec<MinFieldsSession>, DbError>;
    async fn delete_all_by_user(&self, user_id: &str) -> Result<(), DbError>;
}

pub struct SessionRepositoryPg { pub pool: Arc<PgPool>  }

#[async_trait::async_trait]
impl SessionRepository for SessionRepositoryPg {
    async fn create(&self, session: &Session) -> Result<Session, DbError> {
        match sqlx::query_as::<_, Session>("INSERT INTO sessions (id, user_id, refresh_token, device_info, status, expires_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(session.id)
            .bind(session.user_id)
            .bind(&session.refresh_token)
            .bind(&session.device_info)
            .bind(&session.status)
            .bind(session.expires_at)
            .fetch_one(&*self.pool)
            .await
        {
            Ok(sess) => Ok(sess),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_by_session_id(&self, session_id: &str) -> Result<Option<Session>, DbError> {
        let session_id = sqlx::types::Uuid::parse_str(session_id)?;
        match sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_optional(&*self.pool)
            .await
        {
            Ok(session) => Ok(session),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn update_with_refresh(&self, session_id: &str, refresh_token: &str) -> Result<Session, DbError> {
        let session_id = sqlx::types::Uuid::parse_str(session_id)?;

        let res =  sqlx::query_as::<_, Session>("UPDATE sessions SET refresh_token = $1 WHERE id = $2 RETURNING *")
            .bind(refresh_token)
            .bind(session_id)
            .fetch_one(&*self.pool)
            .await;

        match res {
            Ok(session) => Ok(session),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn terminate_with_status(&self, session_id: &str, session_status: SessionStatus) -> Result<Session, DbError> {
        let session_id = sqlx::types::Uuid::parse_str(session_id)?;

        let res = sqlx::query_as::<_, Session>("UPDATE sessions SET status = $1 WHERE id = $2 AND status = $3 RETURNING *")
            .bind(session_status)
            .bind(session_id)
            .bind(SessionStatus::Active)
            .fetch_one(&*self.pool)
            .await;

        match res {
            Ok(sess) => Ok(sess),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn terminate_others_with_status(&self, user_id: &str, session_id: &str, session_status: SessionStatus) -> Result<Vec<Session>, DbError> {
        let session_id = sqlx::types::Uuid::parse_str(session_id)?;
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;

        let res = sqlx::query_as::<_, Session>("UPDATE sessions SET status = $1 WHERE user_id = $2 AND id != $3 AND status = $4 RETURNING *")
            .bind(session_status)
            .bind(user_id)
            .bind(session_id)
            .bind(SessionStatus::Active)
            .fetch_all(&*self.pool)
            .await;

        match res {
            Ok(sess) => Ok(sess),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_all_by_user_full(&self, user_id: &str) -> Result<Vec<Session>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&*self.pool)
            .await
        {
            Ok(sessions) => Ok(sessions),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_all_by_user_min(&self, user_id: &str) -> Result<Vec<MinFieldsSession>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, MinFieldsSession>("SELECT id, device_info, status, expires_at, updated_at, created_at FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&*self.pool)
            .await
        {
            Ok(sessions) => Ok(sessions),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn delete_all_by_user(&self, user_id: &str) -> Result<(), DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}