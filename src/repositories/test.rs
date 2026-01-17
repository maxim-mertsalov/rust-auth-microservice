use std::sync::Arc;
use redis::AsyncCommands;
use crate::db::postgres::PgPool;
use crate::db::redis::RedisPool;
use crate::errors::db_error::DbError;

pub struct TestRepository {
    pg_pool: Arc<PgPool>,
    redis_pool: Arc<RedisPool>
}

impl TestRepository {
    pub fn new(pg_pool: Arc<PgPool>, redis_pool: Arc<RedisPool>) -> Self {
        TestRepository {
            pg_pool,
            redis_pool
        }
    }
    pub async fn test_postgres(&self) -> Result<String, DbError> {
        let row: Result<(String,), sqlx::Error> = sqlx::query_as("SELECT 'Postgres works fine'")
            .fetch_one(&*self.pg_pool)
            .await;

        match row {
            Ok((val,)) => Ok(val),
            Err(e) => Err(DbError::from(e)),
        }
    }

    pub async fn test_redis(&self) -> Result<String, DbError> {
        let mut conn = self.redis_pool.get().await?;

        let _: () = conn.set("test_key", "Redis works fine").await?;
        let val = conn.get("test_key").await?;
        let _: () = conn.del("test_key").await?;
        Ok(val)
    }
}