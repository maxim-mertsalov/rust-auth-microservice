use serde::{Deserialize, Serialize};


#[derive(Debug, Clone)]
pub struct TokenCreatorParams {
    pub user_id: sqlx::types::Uuid,
    pub days_to_inactive: i32,
}


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum Scope {
    #[serde(rename = "openid")]
    OpenId, // For access token
    #[serde(rename = "profile")]
    Profile, // For basic profile information
    #[serde(rename = "email")]
    Email, // For email information
    #[serde(rename = "offline_access")]
    OfflineAccess, // For refresh tokens
}

impl Scope {
    pub fn one_of(scope: &String) -> Option<Scope> {
        match scope.as_ref() {
            "openid" => Some(Scope::OpenId),
            "profile" => Some(Scope::Profile),
            "email" => Some(Scope::Email),
            "offline_access" => Some(Scope::OfflineAccess),
            _ => None,
        }
    }
}