use std::sync::Arc;
use crate::db::postgres::PgPool;
use crate::db::redis::RedisPool;
use crate::repositories::auth::sessions::{SessionRepositoryPg};
use crate::repositories::auth::signin::SignInRepositoryRedis;
use crate::repositories::auth::signup::{SignUpRepositoryRedis};
use crate::repositories::auth::tokens::{TokenSessionRepositoryRedis};
use crate::repositories::auth::users::{UserRepositoryPg};

pub mod users;
pub mod sessions;
pub mod tokens;
pub mod reset_auth;
pub mod signup;
pub mod signin;
mod recovery_emails;
mod recovery_codes;

pub struct AuthRepositories {
    pub users_repo: UserRepositoryPg,
    pub tokens_repo: TokenSessionRepositoryRedis,
    pub sessions_repo: SessionRepositoryPg,
    pub signup_repo: SignUpRepositoryRedis,
    pub signin_repo: SignInRepositoryRedis
}

impl AuthRepositories {
    pub fn new(pg_pool: Arc<PgPool>, redis_pool: Arc<RedisPool>) -> Self {
        let shared_pg_pool = pg_pool;
        let shared_redis_pool = redis_pool;

        Self {
            users_repo: UserRepositoryPg { pool: shared_pg_pool.clone() },
            tokens_repo: TokenSessionRepositoryRedis { pool: shared_redis_pool.clone() },
            sessions_repo: SessionRepositoryPg { pool: shared_pg_pool },
            signup_repo: SignUpRepositoryRedis { pool: shared_redis_pool.clone() },
            signin_repo: SignInRepositoryRedis { pool: shared_redis_pool  },
        }
    }
}