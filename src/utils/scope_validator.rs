use log::info;
use crate::models::auth::utils::Scope;

pub fn validate_scopes(scopes: &Vec<String>) -> Vec<Scope> {
    let mut parsed_scopes: Vec<Scope> = Vec::new();

    if scopes.is_empty() {
        parsed_scopes.push(Scope::Profile); // default scope
        return parsed_scopes;
    }

    if scopes[0].starts_with("full_access") {
        return vec![Scope::Profile, Scope::Email, Scope::OpenId, Scope::OfflineAccess];
    }

    for scope in scopes {
        if let Some(scope_parsed) = Scope::one_of(scope) {
            parsed_scopes.push(scope_parsed);
        }
    }

    parsed_scopes.sort();
    parsed_scopes.dedup();

    parsed_scopes
}