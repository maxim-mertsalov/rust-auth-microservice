use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RecoveryEmails {
    pub id: sqlx::types::Uuid,
    pub user_id: sqlx::types::Uuid,
    pub recovery_email: String,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}