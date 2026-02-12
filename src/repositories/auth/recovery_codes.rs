use std::sync::Arc;
use crate::db::postgres::PgPool;
use crate::errors::db_error::DbError;
use crate::models::auth::recovery_codes::RecoveryCodes;

#[async_trait::async_trait]
pub trait RecoveryCodesRepository {
    async fn create(&self, user_id: &str, recovery_codes: Vec<&str>) -> Result<Vec<RecoveryCodes>, DbError>;
    async fn get_by_user_id_and_code(&self, user_id: &str, recovery_code: &str) -> Result<Option<RecoveryCodes>, DbError>;
    async fn get_count_of_codes(&self, user_id: &str) -> Result<i64, DbError>;
    async fn exists(&self, user_id: &str) -> Result<bool, DbError>;
    async fn delete_by_id(&self, recovery_id: &str) -> Result<(), DbError>;
    async fn delete_all_by_user(&self, user_id: &str) -> Result<(), DbError>;
}

pub struct RecoveryCodesRepositoryPg { pub pool: Arc<PgPool>  }

#[async_trait::async_trait]
impl RecoveryCodesRepository for RecoveryCodesRepositoryPg {
    async fn create(&self, user_id: &str, recovery_codes: Vec<&str>) -> Result<Vec<RecoveryCodes>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;

        match sqlx::query_as::<_, RecoveryCodes>(
            "INSERT INTO user_recovery_codes (user_id, recovery_code) SELECT $1, * FROM unnest($2) RETURNING *"
        )
            .bind(user_id)
            .bind(&recovery_codes[..])
            .fetch_all(&*self.pool)
            .await
        {
            Ok(data) => Ok(data),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_by_user_id_and_code(&self, user_id: &str, recovery_code: &str) -> Result<Option<RecoveryCodes>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, RecoveryCodes>("SELECT * FROM user_recovery_codes WHERE user_id = $1 AND recovery_code = $2")
            .bind(user_id)
            .bind(recovery_code)
            .fetch_optional(&*self.pool)
            .await
        {
            Ok(sess) => Ok(sess),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_count_of_codes(&self, user_id: &str) -> Result<i64, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_recovery_codes WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, user_id: &str) -> Result<bool, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_recovery_codes WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(exists)
    }

    async fn delete_by_id(&self, recovery_id: &str) -> Result<(), DbError> {
        let recovery_id = sqlx::types::Uuid::parse_str(recovery_id)?;

        let num = sqlx::query("DELETE FROM user_recovery_codes WHERE id = $1")
            .bind(recovery_id)
            .execute(&*self.pool)
            .await?;

        if num.rows_affected() == 0 {
            return Err(DbError::NotFound("No recovery codes found".to_string()));
        }

        Ok(())
    }

    async fn delete_all_by_user(&self, user_id: &str) -> Result<(), DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        let num = sqlx::query("DELETE FROM user_recovery_codes WHERE user_id = $1")
            .bind(user_id)
            .execute(&*self.pool)
            .await?;

        if num.rows_affected() == 0 {
            return Err(DbError::NotFound("No recovery codes found for the user".to_string()));
        }

        Ok(())
    }
}