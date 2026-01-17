use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use tokio::task;
use crate::errors::hash_error::HashError;

pub struct Hasher;

impl Hasher {
    pub async fn hash_password(password: String) -> Result<String, HashError> {
        // We use tokio::task::spawn_blocking to offload CPU work
        // This keeps the Service async-friendly but handles blocking internally
        task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();

            argon2
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
                .map_err(|e| HashError::from(e))
        })
            .await.map_err(|e| HashError::Other(format!("Thread pool error: {}", e)))?
    }

    pub async fn verify_password(hash: String, password: String) -> Result<bool, HashError> {
        task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash)?;
            let argon2 = Argon2::default();

            Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
        })
            .await
            .map_err(|e| HashError::Other(format!("Thread pool error: {}", e)) )?
    }
}