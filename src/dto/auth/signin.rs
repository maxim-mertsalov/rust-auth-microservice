use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::models::auth::sessions::DeviceInfo;
use crate::models::auth::signin::SignInState;


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