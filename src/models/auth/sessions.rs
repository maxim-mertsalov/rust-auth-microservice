use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Device information structure
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, Default, PartialEq)]
pub struct DeviceInfo {
    pub ip_address: String,
    pub user_agent: String,
}

/// Enum for session status
#[derive(sqlx::Type, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[sqlx(type_name = "session_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionStatus {
    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "TERMINATED_BY_USER")]
    TerminatedByUser,

    #[serde(rename = "TERMINATED_BY_ADMIN")]
    TerminatedByAdmin,

    #[serde(rename = "COMPROMISED")]
    Compromised,

    #[serde(rename = "EXPIRED")]
    Expired,
}

pub const TERMINATED_SESSION_EXPIRATION_DAYS: i64 = 7;

/// sessions table
#[derive(Debug, Serialize, FromRow)]
pub struct Session {
    pub id: sqlx::types::Uuid,
    pub user_id: sqlx::types::Uuid,
    pub refresh_token: String,
    pub device_info: sqlx::types::Json<DeviceInfo>,
    pub status: SessionStatus,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MinFieldsSession {
    pub id: sqlx::types::Uuid,
    pub device_info: sqlx::types::Json<DeviceInfo>,
    pub status: SessionStatus,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}