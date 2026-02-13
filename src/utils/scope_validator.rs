use log::info;
use crate::models::auth::sudo_tokens::SudoTokenScope;
use crate::models::auth::utils::Scope;

pub struct ScopeValidationResult {
    pub scopes: Vec<Scope>,
    pub sudo_scope: Option<SudoTokenScope>
}


pub fn validate_scopes(scopes: &Vec<String>) -> ScopeValidationResult {
    let mut result = ScopeValidationResult {
        scopes: Vec::new(),
        sudo_scope: None,
    };

    if scopes.is_empty() {
        result.scopes.push(Scope::Profile); // default scope
        return result;
    }

    if scopes[0].starts_with("full_access") {
        result.scopes = vec![Scope::Profile, Scope::Email, Scope::OpenId, Scope::OfflineAccess];
        return result;
    }

    if scopes[0].starts_with("sudo_mode") {
        let sudo_scope_str = scopes[0].split(":").nth(1).unwrap_or("");
        let sudo_scope = serde_json::from_str::<SudoTokenScope>(&format!("\"{}\"", sudo_scope_str));
        if let Ok(sudo_scope_parsed) = sudo_scope {
            result.scopes.push(Scope::SudoMode);
            result.sudo_scope = Some(sudo_scope_parsed);
        } else {
            result.sudo_scope = None;
        }

        return result
    }

    for scope in scopes {
        if let Some(scope_parsed) = Scope::one_of(scope) {
            result.scopes.push(scope_parsed);
        }
    }

    result.scopes.sort();
    result.scopes.dedup();

    result
}