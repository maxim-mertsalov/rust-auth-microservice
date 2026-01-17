use serde::{Deserialize, Serialize};
use crate::models::auth::sessions::DeviceInfo;

/// session:{hashed_refresh_token}
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenSession {
    pub session_id: String,
    pub user_id: String,
    pub device_info: DeviceInfo,
    pub expires_in: i32, // in days
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub const TOKEN_SESSION_PREFIX: &str = "session";
// pub const TOKEN_SESSION_LIFETIME: i64 = 15; - taken from the postgres session expiry

/// access_token
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String, // user_id
    pub session_id: String, // session_id
    pub exp: usize, // expiration time as unix timestamp
    pub iat: usize, // issued at as unix timestamp
}

pub const ACCESS_TOKEN_EXPIRY_MINUTES: i64 = 5;

