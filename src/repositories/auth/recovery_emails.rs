use std::sync::Arc;
use crate::db::postgres::PgPool;
use crate::errors::db_error::DbError;
use crate::models::auth::recovery_emails::RecoveryEmails;

#[async_trait::async_trait]
pub trait RecoveryEmailsRepository {
    async fn create(&self, user_id: &str, recovery_email: &str) -> Result<RecoveryEmails, DbError>;
    async fn verify_email(&self, email_id: &str) -> Result<Option<RecoveryEmails>, DbError>;
    async fn get_by_user_id(&self, user_id: &str) -> Result<Vec<RecoveryEmails>, DbError>;
    async fn get_by_user_id_verified(&self, user_id: &str) -> Result<Vec<RecoveryEmails>, DbError>;
    async fn get_count_of_emails(&self, user_id: &str) -> Result<i64, DbError>;
    async fn exists_verified(&self, user_id: &str) -> Result<bool, DbError>;
    async fn delete_with_id(&self, email_id: &str) -> Result<(), DbError>;
}

pub struct RecoveryEmailRepositoryPg { pub pool: Arc<PgPool>  }

#[async_trait::async_trait]
impl RecoveryEmailsRepository for RecoveryEmailRepositoryPg {
    async fn create(&self, user_id: &str, recovery_email: &str) -> Result<RecoveryEmails, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, RecoveryEmails>(
            "INSERT INTO user_recovery_emails (user_id, recovery_email) VALUES ($1, $2) RETURNING *"
        )
            .bind(user_id)
            .bind(recovery_email)
            .fetch_one(&*self.pool)
            .await
        {
            Ok(data) => Ok(data),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn verify_email(&self, email_id: &str) -> Result<Option<RecoveryEmails>, DbError> {
        let email_id = sqlx::types::Uuid::parse_str(email_id)?;

        let res =  sqlx::query_as::<_, RecoveryEmails>("UPDATE user_recovery_emails SET verified_at = now() WHERE id = $1 RETURNING *")
            .bind(email_id)
            .fetch_optional(&*self.pool)
            .await;

        match res {
            Ok(email) => Ok(email),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_by_user_id(&self, user_id: &str) -> Result<Vec<RecoveryEmails>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, RecoveryEmails>("SELECT * FROM user_recovery_emails WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&*self.pool)
            .await
        {
            Ok(emails) => Ok(emails),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_by_user_id_verified(&self, user_id: &str) -> Result<Vec<RecoveryEmails>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, RecoveryEmails>("SELECT * FROM user_recovery_emails WHERE user_id = $1 AND verified_at IS NOT NULL")
            .bind(user_id)
            .fetch_all(&*self.pool)
            .await
        {
            Ok(emails) => Ok(emails),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_count_of_emails(&self, user_id: &str) -> Result<i64, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_recovery_emails WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(count)
    }

    async fn exists_verified(&self, user_id: &str) -> Result<bool, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_recovery_emails WHERE user_id = $1 AND verified_at IS NOT NULL)")
            .bind(user_id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(exists)
    }

    async fn delete_with_id(&self, email_id: &str) -> Result<(), DbError> {
        let id = sqlx::types::Uuid::parse_str(email_id)?;

        let num = sqlx::query("DELETE FROM user_recovery_emails WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await?;

        if num.rows_affected() == 0 {
            return Err(DbError::NotFound("No recovery codes found".to_string()));
        }

        Ok(())
    }
}