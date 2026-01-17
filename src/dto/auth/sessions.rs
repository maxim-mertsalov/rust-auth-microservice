use serde::{Deserialize, Serialize};
use crate::models::auth::sessions::DeviceInfo;
//# ----- Tokens -----

// Request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenReq {
    pub device_info: DeviceInfo,
    pub access_token: String,
    pub refresh_token: String,
}

// Response
#[derive(Debug, Serialize)]
pub struct RefreshTokenRes {
    pub access_token: String,
    pub refresh_token: String,
}


//# ----- Check token -----
// Request
#[derive(Debug, Deserialize)]
pub struct CheckTokenReq {
    pub access_token: String,
}


//# ----- Logout -----
// Request
#[derive(Debug, Deserialize)]
pub struct LogoutReq {
    pub access_token: String,
    pub refresh_token: String,
}


//# ----- Get All Sessions -----
// Request
#[derive(Debug, Deserialize)]
pub struct GetAllSessionsReq {
    pub access_token: String,
}

// Response
// Note: reusing MinFieldsSession from models


//# ----- Terminate Session -----
// Request
#[derive(Debug, Deserialize)]
pub struct TerminateSessionReq {
    pub access_token: String,

    pub session_id: String,
}


//# ----- Terminate All Other Sessions -----
// Request
#[derive(Debug, Deserialize)]
pub struct TerminateAllOtherSessionsReq {
    pub access_token: String,
}

