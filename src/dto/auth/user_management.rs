use serde::{Deserialize, Serialize};

//# ----- Delete user by id -----
// Request
#[derive(Debug, Deserialize)]
pub struct DeleteUserReq {
    pub access_token: String,
    pub refresh_token: String,
}


//# ----- Update user -----
// Request
#[derive(Debug, Deserialize)]
pub struct UpdateUserReq {
    pub access_token: String,

    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

