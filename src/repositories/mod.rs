use std::sync::Arc;
use crate::db::postgres::PgPool;
use crate::db::redis::RedisPool;
use crate::repositories::auth::AuthRepositories;
pub(crate) use crate::repositories::test::TestRepository;

pub mod auth;
pub mod test;

pub struct AppRepositories {
    pub auth_repos: AuthRepositories,
    pub test_repo: TestRepository
}

impl AppRepositories {
    pub fn new(pg_pool: Arc<PgPool>, redis_pool: Arc<RedisPool>) -> Self {
        let shared_pg_pool = pg_pool;
        let shared_redis_pool = redis_pool;

        AppRepositories {
            auth_repos: AuthRepositories::new(shared_pg_pool.clone(), shared_redis_pool.clone()),
            test_repo: TestRepository::new(shared_pg_pool, shared_redis_pool)
        }

    }
}