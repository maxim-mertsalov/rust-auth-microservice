use log::{info};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use crate::errors::db_error::DbError;

pub type PgPool = Pool<Postgres>;

pub async fn create_postgres_pool(db_url: &str, max_connections: u32) -> Result<PgPool, DbError> {
   let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(db_url)
        .await;

    match pool {
        Ok(pool) => {
            info!("Postgres connection pool created successfully");
            Ok(pool)
        }
        Err(err) => {
            Err(DbError::from(err))
        }
    }
}