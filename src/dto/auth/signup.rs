use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::models::auth::sessions::DeviceInfo;
use crate::models::auth::signup::SignUpState;

//# --- initialisation route ---
// Request
#[derive(Debug, Deserialize)]
pub struct InitSessionReq {
    pub device_info: DeviceInfo,
    pub scopes: Vec<String>,
    pub final_redirect_url: Option<String>,
}

// Response
#[derive(Debug, Serialize)]
pub struct InitSessionRes {
    pub session_token: String,
    pub next_stage: SignUpState,
}


//# --- initialise with name and last name ---
// Request
#[derive(Debug, Deserialize)]
pub struct InitProfileReq {
    pub device_info: DeviceInfo,
    pub scopes: Vec<String>,
    pub final_redirect_url: Option<String>,

    pub first_name: String,
    pub last_name: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct InitProfileRes {
    pub session_token: String,
    pub next_stage: SignUpState,
}


//# --- set name and last name ---
// Request
#[derive(Debug, Deserialize)]
pub struct SetProfileReq {
    pub session_token: String,
    pub first_name: String,
    pub last_name: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct SetProfileRes {
    pub next_stage: SignUpState,
}


//# --- set email ---
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct SetEmailReq {
    pub session_token: String,

    #[validate(email)]
    pub email: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct SetEmailRes {
    pub next_stage: SignUpState,
}


//# --- resend email confirmation ---
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct ResendEmailCodeReq {
    pub session_token: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct ResendEmailCodeRes;


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
    pub next_stage: SignUpState,
}


//# --- set password ---
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct SetPasswordReq {
    pub session_token: String,

    #[validate(custom(function = crate::utils::validators::validate_password), length(min = 8, max = 128))]
    pub password: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct SetPasswordRes {
    pub next_stage: SignUpState,
}


//# --- finalize signup (redirect) ---
// Request
#[derive(Debug, Deserialize)]
pub struct FinalizeSignUpReq {
    pub session_token: String,
}

// Response
#[derive(Debug, Serialize, Default)]
pub struct FinalizeSignUpRes {
    pub redirect_url: Option<String>, // always

    pub access_token: Option<String>, // openid

    pub refresh_token: Option<String>, // offline_access

    pub email: Option<String>, // email

    pub first_name: Option<String>, // profile
    pub last_name: Option<String>, // profile
}


//# --- return back session state ---
// Request
#[derive(Debug, Deserialize)]
pub struct ReturnBackSessionReq {
    pub session_token: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct ReturnBackSessionRes {
    pub next_stage: SignUpState,
}