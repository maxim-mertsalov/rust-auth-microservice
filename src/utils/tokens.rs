use std::time::{SystemTime, UNIX_EPOCH};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header};
use rand::distr;
use rand::distr::{SampleString};
use crate::errors::app_error::AppError;
use crate::models::auth::tokens::{AccessTokenClaims, ACCESS_TOKEN_EXPIRY_MINUTES};

pub struct TokenBuilder;

impl TokenBuilder {
    pub fn encode_access_token(user_id: String, session_id: String, secret: &String) -> Result<String, AppError> {
        let claims = AccessTokenClaims {
            sub: user_id,
            session_id,
            exp: (chrono::Utc::now() + chrono::Duration::minutes(ACCESS_TOKEN_EXPIRY_MINUTES)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))?;

        Ok(token)
    }

    /// Decodes an access token and checks its expiration status.
    ///
    /// Returns: `Result<(AccessTokenClaims, bool), AppError>`
    /// - `(claims, false)`: Token is **valid** and not expired.
    /// - `(claims, true)`: Token is **expired**, but claims were successfully read.
    /// - `Err(AppError)`: Token is invalid for reasons other than expiration (e.g., signature, format).
    pub fn decode_access_token(token: &str, secret: &str) -> Result<(AccessTokenClaims, bool), AppError> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = false;
        validation.set_required_spec_claims(&["sub", "session_id", "exp", "iat"]);

        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        match jsonwebtoken::decode::<AccessTokenClaims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;

                let current_time_seconds: usize = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| AppError::InternalServerError("Time went backwards".to_string()))?
                    .as_secs().try_into().map_err(|_| AppError::InternalServerError("Time parse error".to_string()))?;

                if claims.exp < current_time_seconds {
                    Ok((claims, true))
                } else {
                    Ok((claims, false))
                }
            }

            Err(error) => Err(AppError::from(error))
        }
    }

    pub fn generate_refresh_token() -> String {
        distr::Alphanumeric.sample_string(&mut rand::rng(), 50)
    }
}