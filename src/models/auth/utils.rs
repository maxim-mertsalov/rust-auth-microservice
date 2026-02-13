use serde::{Deserialize, Serialize};
use crate::models::auth::sessions::DeviceInfo;
use crate::models::auth::sudo_tokens::SudoTokenScope;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AuthFlowMetadata {
    pub scopes: Vec<Scope>,
    pub final_redirect_url: Option<String>,
    pub device_info: DeviceInfo,
    pub sudo_scope: Option<SudoTokenScope>,
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
    #[serde(rename = "sudo_mode")]
    SudoMode, // For security-sensitive operations, requires recent authentication. Disallowed in signup flow.
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