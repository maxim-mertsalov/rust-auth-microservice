use serde::{Deserialize, Serialize};
use crate::models::auth::sessions::DeviceInfo;
use crate::models::auth::utils::Scope;

/// password_reset:{session_token}
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordReset {
    // pub token: String,

    pub scopes: Vec<Scope>,
    pub final_redirect_url: Option<String>,

    pub data: PwdResetData,

    pub device_info: DeviceInfo,

    pub stage: PwdResetState,

    pub otp_code: String,
    pub attempts: u8,
}

pub const PASSWORD_RESET_PREFIX: &str = "password_reset";
pub const PASSWORD_RESET_LIFETIME: i64 = 15;


#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum PwdResetState {
    #[default]
    InitWithEmailStage,
    EmailVerificationStage,

    SetPasswordStage,
    Redirect
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PwdResetData {
    pub user_id: Option<String>,
    pub last_resent_code_at: Option<chrono::DateTime<chrono::Utc>>,
}