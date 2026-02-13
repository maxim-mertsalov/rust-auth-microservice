use serde::{Deserialize, Serialize};

/// access_token
#[derive(Debug, Serialize, Deserialize)]
pub struct SudoTokenClaims {
    pub sub: String, // user_id
    pub scope: SudoTokenScope,
    pub exp: usize, // expiration time as unix timestamp
    pub iat: usize, // issued at as unix timestamp
}

pub const SUDO_TOKEN_EXPIRY_MINUTES: i64 = 10;


#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum SudoTokenScope {
    #[serde(rename = "change_email")]
    ChangeEmail,
    #[serde(rename = "change_password")]
    ChangePassword,
    #[serde(rename = "delete_account")]
    DeleteAccount,
    #[serde(rename = "add_recovery_email")]
    AddRecoveryEmail,
    #[serde(rename = "remove_recovery_email")]
    RemoveRecoveryEmail,
    #[serde(rename = "generate_recovery_codes")]
    GenerateRecoveryCodes,
    #[serde(rename = "revoke_sessions")]
    RevokeSessions,
    #[serde(rename = "all")]
    All
}