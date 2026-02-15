use std::sync::Arc;
use crate::dto::auth::sessions::{GetAllSessionsReq, LogoutReq, RefreshTokenReq, RefreshTokenRes, TerminateAllOtherSessionsReq, TerminateSessionReq};
use crate::errors::app_error::AppError;
use crate::models::auth::sessions::{MinFieldsSession, Session, SessionStatus, TERMINATED_SESSION_EXPIRATION_DAYS};
use crate::repositories::auth::{AuthRepositories};
use crate::repositories::auth::sessions::SessionRepository;
use crate::repositories::auth::tokens::TokenSessionRepository;
use crate::state::app_state::AppState;
use crate::utils::converter::Converter;
use crate::utils::access_tokens::TokenBuilder;

#[async_trait::async_trait]
pub trait ISessionService {
    /// ----- Refresh tokens by /refresh -----
    async fn refresh_tokens(&self, app_state: &AppState, req: RefreshTokenReq) -> Result<RefreshTokenRes, AppError>;

    /// ----- Check user by /auth/check
    async fn check_user(&self, app_state: &AppState, access_token: String) -> Result<bool, AppError>;

    /// ----- Logout user by /auth/logout -----
    async fn logout_user(&self, app_state: &AppState, req: LogoutReq) -> Result<(), AppError>;

    async fn get_all_sessions(&self, app_state: &AppState, req: GetAllSessionsReq) -> Result<Vec<MinFieldsSession>, AppError>;
    async fn terminate_session(&self, app_state: &AppState, req: TerminateSessionReq) -> Result<(), AppError>;
    async fn terminate_all_other_sessions(&self, app_state: &AppState, req: TerminateAllOtherSessionsReq) -> Result<(), AppError>;
}

pub struct SessionService {
    repos: Arc<AuthRepositories>,
}

#[async_trait::async_trait]
impl ISessionService for SessionService {
    async fn refresh_tokens(&self, app_state: &AppState, req: RefreshTokenReq) -> Result<RefreshTokenRes, AppError> {
        // 1. Validate the access token
        // Decode the access token to get claims
        let (token_data, exp) = TokenBuilder::decode_access_token(&req.access_token, app_state.get_secret_key())?;

        if !exp {
            return Err(AppError::BadRequest("Token is not expired".to_string()));
        }

        let refresh_token_hash = Converter::hash_string(&req.refresh_token);


        // 2. Check if the session exists and is active
        let session = match self.repos.tokens_repo.get(&refresh_token_hash).await? {
            Some(sess) => {
                sess
            },
            None => {
                let session_data = self.repos.sessions_repo.get_by_session_id(&token_data.session_id).await?
                    .ok_or_else(|| AppError::Unauthorized("Session not found".to_string()))?;

                if session_data.status == SessionStatus::Active {
                    // in this session we have already hashed refresh token
                    let _ = self.repos.tokens_repo.delete(&session_data.refresh_token).await?;

                    //TODO! RabbitMQ -> TERMINATE WITH STATUS
                    let _ = self.repos.sessions_repo.terminate_with_status(&token_data.session_id, SessionStatus::Compromised).await?;
                }

                return Err(AppError::Unauthorized("Invalid tokens".to_string()))
            },
        };


        // 3. Fetch session from Postgres to verify its status
        let session_data = self.repos.sessions_repo.get_by_session_id(&token_data.session_id).await?;
        match session_data {
            Some(sess) => {
                if sess.status != SessionStatus::Active {
                    let _ = self.repos.tokens_repo.delete(&refresh_token_hash).await?;
                    return Err(AppError::Unauthorized("Session is not active".to_string()));
                }
            },
            None => {
                let _ = self.repos.tokens_repo.delete(&refresh_token_hash).await?;
                return Err(AppError::Unauthorized("Session not found".to_string()));
            },
        };

        // 4. Check if device info matches
        let user_agent_check = session.device_info.user_agent != req.device_info.user_agent;

        // 4.0 All checks
        if user_agent_check {
            self.repos.tokens_repo.delete(&refresh_token_hash).await?;

            //TODO! RabbitMQ -> TERMINATE WITH STATUS
            self.repos.sessions_repo.terminate_with_status(&token_data.session_id, SessionStatus::Compromised).await?;

            return Err(AppError::Unauthorized("Device information does not match".to_string()));
        }

        // 5. Generate new tokens
        let new_refresh_token = TokenBuilder::generate_refresh_token();
        let new_refresh_token_hash = Converter::hash_string(&new_refresh_token);

        let new_access_token = TokenBuilder::encode_access_token(session.user_id.clone(), session.session_id.clone(), app_state.get_secret_key())?;

        let mut new_token_session = session;
        new_token_session.updated_at = chrono::Utc::now();
        new_token_session.expires_at = chrono::Utc::now() + chrono::Duration::days(new_token_session.expires_in as i64);

        // 6. Update session in DB and Redis with new refresh token
        let _ = self.repos.tokens_repo.delete(&refresh_token_hash).await?;
        let _ = self.repos.tokens_repo.create(&new_refresh_token_hash, &new_token_session).await?;

        //TODO! RabbitMQ -> UPDATE REFRESH TOKEN
        let _ = self.repos.sessions_repo.update_with_refresh(&new_token_session.session_id, &new_refresh_token_hash).await?;

        Ok(RefreshTokenRes {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
        })
    }

    /// returns `true` if user is expired or `false` if user is not expired
    async fn check_user(&self, app_state: &AppState, access_token: String) -> Result<bool, AppError> {
        let (_, exp) = TokenBuilder::decode_access_token(&access_token, app_state.get_secret_key())?;

        Ok(exp)
    }

    async fn logout_user(&self, app_state: &AppState, req: LogoutReq) -> Result<(), AppError> {
        let (access_token_data, exp) = TokenBuilder::decode_access_token(&req.access_token, app_state.get_secret_key())?;

        if exp {
            return Err(AppError::ExpiredAccessToken);
        }

        let hashed_refresh_token = Converter::hash_string(&req.refresh_token);

        if self.repos.tokens_repo.get(&hashed_refresh_token).await?.is_none() {
            let session = self.repos.sessions_repo.get_by_session_id(&access_token_data.session_id).await?
                .ok_or_else(|| AppError::BadRequest("Session not found".to_string()))?;

            if session.status == SessionStatus::Active {
                let _ = self.repos.tokens_repo.delete(&session.refresh_token).await?;

                //TODO! RabbitMQ -> TERMINATE WITH STATUS
                let _ = self.repos.sessions_repo.terminate_with_status(&access_token_data.session_id, SessionStatus::TerminatedByUser).await?;
                return Ok(())
            }
        }

        let _ = self.repos.tokens_repo.delete(&hashed_refresh_token).await?;

        //TODO! RabbitMQ -> TERMINATE WITH STATUS
        let _ = self.repos.sessions_repo.terminate_with_status(&access_token_data.session_id, SessionStatus::TerminatedByUser).await?;

        Ok(())
    }

    async fn get_all_sessions(&self, app_state: &AppState, req: GetAllSessionsReq) -> Result<Vec<MinFieldsSession>, AppError> {
        let (token_data, exp) = TokenBuilder::decode_access_token(&req.access_token, app_state.get_secret_key())?;

        if exp {
            return Err(AppError::ExpiredAccessToken);
        }

        let sessions = self.repos.sessions_repo.get_all_minimised_by_user_id(&token_data.sub).await?;

        Ok(sessions)
    }

    async fn terminate_session(&self, app_state: &AppState, req: TerminateSessionReq) -> Result<(), AppError> {
        let (token_data, exp) = TokenBuilder::decode_access_token(&req.access_token, app_state.get_secret_key())?;

        if exp {
            return Err(AppError::ExpiredAccessToken);
        }

        let session = self.repos.sessions_repo.get_by_session_id(&req.session_id).await?
            .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

        if session.user_id.to_string() != token_data.sub {
            return Err(AppError::Unauthorized("You are not authorized to terminate this session".to_string()));
        }

        if session.status != SessionStatus::Active {
            return Err(AppError::BadRequest("Session is not active".to_string()));
        }

        self.repos.sessions_repo.terminate_with_status(&token_data.session_id, SessionStatus::TerminatedByUser).await?;

        let _ = self.repos.tokens_repo.delete(&session.refresh_token).await;

        Ok(())
    }

    async fn terminate_all_other_sessions(&self, app_state: &AppState, req: TerminateAllOtherSessionsReq) -> Result<(), AppError> {
        let (token_data, exp) = TokenBuilder::decode_access_token(&req.access_token, app_state.get_secret_key())?;

        if exp {
            return Err(AppError::ExpiredAccessToken);
        }

        let res = self.repos.sessions_repo.terminate_others_with_status(&token_data.sub, &token_data.session_id, SessionStatus::TerminatedByUser).await?;

        for session in res {
            let _ = self.repos.tokens_repo.delete(&session.refresh_token).await;
        }

        Ok(())
    }
}

impl SessionService {
    pub fn new(repos: Arc<AuthRepositories>) -> Self {
        Self { repos }
    }
}
