use serde::{Deserialize, Serialize};

// email_verification:{email}
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerification {
    pub user_id: String,
    pub verification_code: String,
    pub attempts: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub const EMAIL_VERIFICATION_LIFETIME: u64 = 10; // minutes
pub const EMAIL_VERIFICATION_PREFIX: &str = "email_verification";
