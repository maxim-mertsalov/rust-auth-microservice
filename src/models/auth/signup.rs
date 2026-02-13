use serde::{Deserialize, Serialize};
use crate::models::auth::sessions::DeviceInfo;
use crate::models::auth::utils::Scope;

/// signup:{token}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SignUpSession {
    // pub token: String,

    pub scopes: Vec<Scope>,
    pub final_redirect_url: Option<String>,

    pub data: SignUpData,

    pub device_info: DeviceInfo,

    // For all verifications (mobile, email, etc)
    pub verification_code: String,
    pub attempts: u8,

    pub stage: SignUpState,
}

pub const SIGNUP_SESSION_PREFIX: &str = "signup";
pub const SIGNUP_SESSION_LIFETIME: u64 = 20; // reset every time when request anything

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum SignUpState {
    #[default]
    InitStage, // just created session
    InitWithProfileStage, // create session with name and last name
    ProfileStage, // enter name and last name if you skip it

    EmailStage,
    EmailVerificationStage,

    PasswordStage,

    // Addiction Security Questions Stage,
    SetQuestionsStage,

    Redirect
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignUpData {
    pub email: Option<String>,
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub last_resent_code_at: Option<chrono::DateTime<chrono::Utc>>,
}