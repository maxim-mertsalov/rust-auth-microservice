use serde::{Deserialize, Serialize};
use crate::models::auth::sessions::DeviceInfo;
use crate::models::auth::utils::Scope;

/// signup:{token}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SignInSession {
    // pub token: String,

    pub scopes: Vec<Scope>,
    pub final_redirect_url: Option<String>,

    pub data: SignInData,

    pub device_info: DeviceInfo,

    // For all verifications (mobile, email, etc)
    pub verification_code: String,
    pub attempts: u8,

    pub stage: SignInState,
}

pub const SIGNIN_SESSION_PREFIX: &str = "signin";
pub const SIGNIN_SESSION_LIFETIME: i64 = 20; // reset every time when request anything

//TODO: expand challenge types in future
#[derive(Serialize, Deserialize, Debug)]
pub enum ChallengeType {
    EmailOtp, // default
    SmsOtp, // not implemented
    AuthenticatorTotp, // not implemented
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum SignInState {
    #[default]
    InitWithEmailPassStage, //TODO: delete

    InitWithEmailStage,

    SetPasswordStage,

    AuthorizeWithEmailCodeStage, //unimplemented: if you want to skip password you can authorise with recovery email code
    AnswerQuestionStage, //unimplemented: security questions

    EmailVerificationStage, // if 2FA is enabled
    Redirect
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignInData {
    pub user_id: Option<String>,
    pub is_two_fa_enabled: bool,
    pub last_resent_code_at: Option<chrono::DateTime<chrono::Utc>>,
}