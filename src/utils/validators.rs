use std::borrow::Cow;
use std::sync::LazyLock;
use regex::Regex;
use validator::{ValidationError, ValidationErrors};

pub static PASSWORD_FORMAT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^[a-zA-Z0-9!@#$%^&()_+\-=.?]*$"#).unwrap()
});

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    // Check for forbidden characters/overall format

    if !PASSWORD_FORMAT.is_match(password) {
        return Err(ValidationError::new("password").with_message(Cow::from("Password contains invalid characters")))
    }

    // Check for required elements (the "At least one" rules)
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("password").with_message(Cow::from("Must contain at least one uppercase letter")))
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new("password").with_message(Cow::from("Must contain at least one lowercase letter")))
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("password").with_message(Cow::from("Must contain at least one digit")))
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new("password").with_message(Cow::from("Must contain at least one special character")))
    }

    Ok(())
}