use std::sync::Arc;
use crate::db::{postgres::PgPool, redis::RedisPool};
use crate::services::AppServices;

/// App state structure, that contains Postgres and Redis connection pools, and the secret key for JWT
/// Call `builder()` method to create object
#[derive(Clone)]
pub struct AppState {
    //* Services
    services: Arc<AppServices>,

    //* Database connection pools
    pg: Arc<PgPool>,
    redis: Arc<RedisPool>,

    //* Secret keys
    secret_key: String, // secret key for signing JWT tokens

    //* Hashing
    cost: u32,

    //* Feature flags
    require_email_confirmation: bool, // if true email_sender confirmation is required after registration
    enable_email_reset: bool, // if true changing emails is enabled
}

/// Builder for AppState
/// Allows setting optional parameters and building the final AppState
/// Use `with_pg`, `with_redis`, `with_secret_key` to set required parameters
/// Use `with_require_email_confirmation`, `with_enable_email_reset` to set optional feature flags
/// All flags default to `true`
/// Call `build()` to create the AppState instance
pub struct AppStateBuilder {
    services: Option<Arc<AppServices>>,

    pg: Option<Arc<PgPool>>,
    redis: Option<Arc<RedisPool>>,

    secret_key: Option<String>,

    cost: u32,
    hash_secret: Option<[u8; 16]>,

    require_email_confirmation: bool,
    enable_email_reset: bool,
}

impl AppStateBuilder {
    fn new() -> Self {
        Self {
            services: None,

            pg: None,
            redis: None,
            
            secret_key: None,

            cost: 8,
            hash_secret: None,

            require_email_confirmation: true,
            enable_email_reset: true,
        }
    }

    pub fn with_services(mut self, services: AppServices) -> Self { self.services = Some(Arc::new(services)); self }

    pub fn with_pg_pool(mut self, pg: Arc<PgPool>) -> Self { self.pg = Some(pg); self }
    pub fn with_redis_pool(mut self, redis: Arc<RedisPool>) -> Self { self.redis = Some(redis); self }

    pub fn with_secret_key(mut self, secret_key: String) -> Self { self.secret_key = Some(secret_key); self }
    pub fn with_cost(mut self, cost: u32) -> Self { self.cost = cost; self }

    pub fn with_require_email_confirmation(mut self, require: bool) -> Self { self.require_email_confirmation = require; self }
    pub fn with_enable_email_reset(mut self, enable: bool) -> Self { self.enable_email_reset = enable; self }


    pub fn build(self) -> Result<AppState, &'static str> {
        Ok(AppState {
            services: self.services.ok_or("Services object is required")?,

            pg: self.pg.ok_or("Postgres pool is required")?,
            redis: self.redis.ok_or("Redis pool is required")?,

            secret_key: self.secret_key.ok_or("Secret key is required")?,

            cost: self.cost,

            require_email_confirmation: self.require_email_confirmation,
            enable_email_reset: self.enable_email_reset,
        })
    }
}

impl AppState {
    /// Get a clone of reference to Services object
    pub fn get_services(&self) -> Arc<AppServices> { self.services.clone() }

    /// Get a reference to the Postgres connection pool
    pub fn get_pg(&self) -> Arc<PgPool> { self.pg.clone() }

    /// Get a reference to the Redis connection pool
    pub fn get_redis(&self) -> Arc<RedisPool> { self.redis.clone() }

    /// Get a reference to the secret keys
    pub fn get_secret_key(&self) -> &String {
        &self.secret_key
    }

    /// Get the cost for password hashing
    pub fn get_cost(&self) -> u32 { self.cost }

    // Feature flag getters
    pub fn is_email_confirmation_required(&self) -> bool { self.require_email_confirmation }
    pub fn is_email_reset_enabled(&self) -> bool { self.enable_email_reset }

    /// Builder
    pub fn builder() -> AppStateBuilder { AppStateBuilder::new() }
}