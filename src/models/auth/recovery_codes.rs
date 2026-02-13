use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Debug, Serialize)]
pub struct RecoveryCodes {
    pub id: sqlx::types::Uuid,
    pub user_id: sqlx::types::Uuid,
    pub recovery_code: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}