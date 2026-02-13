use std::time::{SystemTime, UNIX_EPOCH};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, decode};
use crate::errors::app_error::AppError;
use crate::models::auth::sudo_tokens::{SudoTokenClaims, SudoTokenScope, SUDO_TOKEN_EXPIRY_MINUTES};

pub struct SudoTokenBuilder;


impl SudoTokenBuilder {
    pub fn build_sudo_token(user_id: &String, scope: SudoTokenScope, secret: &String) -> Result<String, AppError> {
        let claims = SudoTokenClaims {
            sub: user_id.clone(),
            scope,
            exp: (chrono::Utc::now() + chrono::Duration::minutes(SUDO_TOKEN_EXPIRY_MINUTES)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))?;

        Ok(token)
    }

    pub fn decode_sudo_token(token: &str, secret: &str) -> Result<(SudoTokenClaims, bool), AppError> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_exp = false;
        validation.set_required_spec_claims(&["sub", "scope", "exp", "iat"]);

        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        match decode::<SudoTokenClaims>(token, &decoding_key, &validation) {
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
}