

#[derive(Debug, Clone)]
pub enum HashError {
    InvalidHash(String),
    HashingFailed(String),
    Other(String),
}

impl From<argon2::password_hash::Error> for HashError {
    fn from(err: argon2::password_hash::Error) -> Self {
        match err {
            argon2::password_hash::Error::Password => HashError::InvalidHash("Invalid password".to_string()),
            _ => HashError::Other(err.to_string()),
        }
    }
}