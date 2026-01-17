use std::sync::Arc;
use sqlx::Postgres;
use crate::db::postgres::PgPool;
use crate::dto::auth::user_management::UpdateUserReq;
use crate::errors::db_error::DbError;
use crate::models::auth::users::{User, MinFieldsUser, MidFieldsUser};

#[async_trait::async_trait]
pub trait UserRepository{
    async fn get_all(&self) -> Result<Vec<MinFieldsUser>, DbError>;
    async fn get_one(&self, user_id: &str) -> Result<Option<MidFieldsUser>, DbError>;
    async fn update_one(&self, user_id: &str, user_req: &UpdateUserReq) -> Result<MidFieldsUser, DbError>;
    async fn create(&self, user: &User) -> Result<User, DbError>;
    async fn get_full_by_email(&self, user_email: &str) -> Result<Option<User>, DbError>;
    async fn get_full_by_id(&self, user_id: &str) -> Result<Option<User>, DbError>;
    async fn delete_by_id(&self, user_id: &str) -> Result<(), DbError>;
}

pub struct UserRepositoryPg{ pub pool: Arc<PgPool>, }

#[async_trait::async_trait]
impl UserRepository for UserRepositoryPg {
    async fn get_all(&self) -> Result<Vec<MinFieldsUser>, DbError> {
        match sqlx::query_as::<_, MinFieldsUser>("SELECT id, first_name, last_name, email FROM users")
            .fetch_all(&*self.pool)
            .await
        {
            Ok(users) => Ok(users),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_one(&self, user_id: &str) -> Result<Option<MidFieldsUser>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, MidFieldsUser>("SELECT id, first_name, last_name, email, is_verified, is_two_factor, updated_at, created_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await
        {
            Ok(user) => Ok(user),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn update_one(&self, user_id: &str, user_req: &UpdateUserReq) -> Result<MidFieldsUser, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;

        let mut query_builder: sqlx::QueryBuilder<Postgres> =
            sqlx::QueryBuilder::new("UPDATE users SET ");

        let mut separated = query_builder.separated(", ");

        if let Some(first) = &user_req.first_name {
            separated.push("first_name = ");
            separated.push_bind_unseparated(first);
        }

        if let Some(last) = &user_req.last_name {
            separated.push("last_name = ");
            separated.push_bind_unseparated(last);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(user_id);
        query_builder.push(" RETURNING id, first_name, last_name, email, is_verified, is_two_factor, updated_at, created_at");

        let res = query_builder.build_query_as::<MidFieldsUser>()
            .fetch_one(&*self.pool)
            .await?;

        Ok(res)
    }

    async fn create(&self, user: &User) -> Result<User, DbError> {
        match sqlx::query_as::<_, User>("INSERT INTO users (id, email, first_name, last_name, password, is_verified) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(user.id)
            .bind(&user.email)
            .bind(&user.first_name)
            .bind(&user.last_name)
            .bind(&user.password)
            .bind(user.is_verified)
            .fetch_one(&*self.pool)
            .await
        {
            Ok(user) => Ok(user),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_full_by_email(&self, user_email: &str) -> Result<Option<User>, DbError> {
        match sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(user_email)
            .fetch_optional(&*self.pool)
            .await
        {
            Ok(user) => Ok(user),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn get_full_by_id(&self, user_id: &str) -> Result<Option<User>, DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await
        {
            Ok(user) => Ok(user),
            Err(e) => Err(DbError::from(e)),
        }
    }

    async fn delete_by_id(&self, user_id: &str) -> Result<(), DbError> {
        let user_id = sqlx::types::Uuid::parse_str(user_id)?;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}