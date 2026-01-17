use serde::{Deserialize, Serialize};
use validator::Validate;

//# ----- Change password -----
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordReq {
    pub access_token: String,

    pub old_password: String,

    #[validate(custom(function = crate::utils::validators::validate_password), length(min = 8, max = 128))]
    pub new_password: String,
}

// Response
pub struct ChangePasswordRes;


//# ----- Forgot password -----
// Request
#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordReq {
    #[validate(email)]
    pub email: String,
}

// Response
pub struct ForgotPasswordRes {
    pub session_token: String,
}

