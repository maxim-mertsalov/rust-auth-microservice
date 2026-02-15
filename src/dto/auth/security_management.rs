use serde::{Deserialize, Serialize};
use validator::Validate;

//# --- Change password ---
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordReq {
    pub sudo_token: String,
    pub access_token: String,

    #[validate(custom(function = crate::utils::validators::validate_password), length(min = 8, max = 128))]
    pub new_password: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct ChangePasswordRes;


//# --- Generate new recovery codes ---
// Request
#[derive(Debug, Deserialize)]
pub struct GenerateRecoveryCodesReq {
    pub sudo_token: String,
    pub access_token: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct GenerateRecoveryCodesRes {
    pub recovery_codes: Vec<String>,
}


//# --- Change email ---
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct ChangeEmailReq {
    pub sudo_token: String,
    pub access_token: String,

    #[validate(email)]
    pub new_email: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct ChangeEmailRes;
//message: "Change email successful. Please verify your new email address to complete the process."


//# --- Verify email ---
// Request
#[derive(Debug, Deserialize)]
pub struct VerifyEmailReq {
    pub access_token: String,

    pub session_token: String,
    pub verification_code: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct VerifyEmailRes;


//# --- Change 2FA status ---
// Request
#[derive(Debug, Deserialize)]
pub struct ChangeTwoFactorStatusReq {
    pub sudo_token: String,
    pub access_token: String,

    pub enable_2fa: bool,
}

// Response
#[derive(Debug, Serialize)]
pub struct ChangeTwoFactorStatusRes;


//# --- Add recovery email ---
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct AddRecoveryEmailReq {
    pub sudo_token: String,
    pub access_token: String,

    #[validate(email)]
    pub recovery_email: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct AddRecoveryEmailRes;


//# --- Verify recovery email ---
// Request
#[derive(Debug, Deserialize)]
pub struct VerifyRecoveryEmailReq {
    pub access_token: String,

    pub verification_code: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct VerifyRecoveryEmailRes;


//# --- Remove recovery email ---
// Request
#[derive(Debug, Deserialize)]
pub struct RemoveRecoveryEmailReq {
    pub sudo_token: String,
    pub access_token: String,

    pub recovery_email: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct RemoveRecoveryEmailRes;