use bb8_redis::{RedisConnectionManager};
use bb8_redis::bb8::Pool;
use log::info;
use crate::errors::db_error::DbError;

pub type RedisPool = Pool<RedisConnectionManager>;

pub async fn create_redis_pool(db_url: &str, max_connections: u32) -> Result<RedisPool, DbError> {
    let manager = RedisConnectionManager::new(db_url)?;

    let pool = Pool::builder()
        .max_size(max_connections)
        .build(manager)
        .await?;

    info!("Redis connection pool created successfully");

    Ok(pool)
}