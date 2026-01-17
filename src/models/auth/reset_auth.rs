use serde::{Deserialize, Serialize};

/// reset_auth:{token}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResetAuth {
    pub user_id: String,
}

pub const RESET_AUTH_PREFIX: &str = "reset_auth";
pub const RESET_AUTH_LIFETIME: i64 = 5;