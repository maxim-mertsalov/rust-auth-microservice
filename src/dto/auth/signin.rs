use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::models::auth::sessions::DeviceInfo;
use crate::models::auth::signin::{AuthenticationMethod, SignInState};


//# --- initialise with email ---
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct InitEmailReq {
    pub device_info: DeviceInfo,
    pub scopes: Vec<String>,
    pub final_redirect_url: Option<String>,

    #[validate(email)]
    pub email: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct InitEmailRes {
    pub session_token: String,
    pub next_stage: SignInState,
}


//# --- get all available authentication methods ---
// Request
#[derive(Debug, Deserialize)]
pub struct GetAuthMethodsReq {
    pub session_token: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct GetAuthMethodsRes {
    pub available_methods: Vec<AuthenticationMethod>,
}


//# --- select authentication method ---
// Request
#[derive(Debug, Deserialize)]
pub struct SelectAuthMethodReq {
    pub session_token: String,
    pub selected_method: AuthenticationMethod,
}

// Response
#[derive(Debug, Serialize)]
pub struct SelectAuthMethodRes {
    pub next_stage: SignInState,
}

//# --- get all recovery emails ---
// Request
#[derive(Debug, Deserialize)]
pub struct GetRecoveryEmailsReq {
    pub session_token: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct GetRecoveryEmailsRes {
    pub recovery_emails: Vec<String>,
}


//# --- select recovery email ---
// Request
#[derive(Debug, Deserialize)]
pub struct SelectRecoveryEmailReq {
    pub session_token: String,
    pub recovery_email_index: usize,
}

// Response
#[derive(Debug, Serialize)]
pub struct SelectRecoveryEmailRes {
    pub next_stage: SignInState,
}


//# --- verify recovery email code ---
// Request
#[derive(Debug, Deserialize)]
pub struct VerifyRecoveryEmailCodeReq {
    pub session_token: String,
    pub verification_code: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct VerifyRecoveryEmailCodeRes {
    pub next_stage: SignInState,
}


//# --- set recovery code ---
// Request
#[derive(Debug, Deserialize)]
pub struct SetRecoveryCodeReq {
    pub session_token: String,
    pub recovery_code: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct SetRecoveryCodeRes {
    pub next_stage: SignInState,
}


//# --- set password ---
// Request
#[derive(Debug, Deserialize)]
pub struct SetPasswordReq {
    pub session_token: String,
    pub password: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct SetPasswordRes {
    pub next_stage: SignInState,
}


//# --- resend code ---
// Request
#[derive(Debug, Deserialize)]
pub struct ResendCodeReq {
    pub session_token: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct ResendCodeRes;


//# --- verify email ---
// Request
#[derive(Debug, Deserialize)]
pub struct VerifyEmailReq {
    pub session_token: String,
    pub verification_code: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct VerifyEmailRes {
    pub next_stage: SignInState,
}


//# --- finalize signin (redirect) ---
// Request
#[derive(Debug, Deserialize)]
pub struct FinalizeSignInReq {
    pub session_token: String,
}

// Response
#[derive(Debug, Serialize, Default)]
pub struct FinalizeSignInRes {
    pub redirect_url: Option<String>,

    pub access_token: Option<String>, // openid

    pub refresh_token: Option<String>, // offline_access

    pub email: Option<String>, // email

    pub first_name: Option<String>, // profile
    pub last_name: Option<String>, // profile
}