use std::sync::Arc;
use log::info;
use crate::dto::auth::utils::TokenCreatorParams;
use crate::errors::app_error::AppError;
use crate::models::auth::sessions::{DeviceInfo, Session, SessionStatus};
use crate::models::auth::tokens::TokenSession;
use crate::models::auth::users::User;
use crate::repositories::auth::{AuthRepositories};
use crate::repositories::auth::sessions::SessionRepository;
use crate::repositories::auth::tokens::TokenSessionRepository;
use crate::state::app_state::AppState;
use crate::utils::converter::Converter;
use crate::utils::access_tokens::TokenBuilder;

#[async_trait::async_trait]
pub trait IAuthUtilsService {
    async fn create_tokens_and_session(
        &self,
        app_state: &AppState,
        user_data: &TokenCreatorParams,
        device_info: DeviceInfo,
    ) -> Result<(String, String), AppError>;
}

#[derive(Clone)]
pub struct AuthUtilsService{
    repos: Arc<AuthRepositories>,
}

#[async_trait::async_trait]
impl IAuthUtilsService for AuthUtilsService {
    /// Create the tokens and session for user
    /// Returns (access_token, refresh_token)
    async fn create_tokens_and_session(&self, app_state: &AppState, user_data: &TokenCreatorParams, device_info: DeviceInfo) -> Result<(String, String), AppError> {
        let refresh_token = TokenBuilder::generate_refresh_token();

        // Hashed refresh token for storing in Postgres and Redis
        let hashed_refresh_token = Converter::hash_string(&refresh_token);

        let session = Session {
            id: uuid::Uuid::now_v7(),
            user_id: user_data.user_id,
            refresh_token: hashed_refresh_token.clone(),
            device_info: sqlx::types::Json::from(device_info.clone()),
            status: SessionStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::days(user_data.days_to_inactive as i64), // 30 days validity
        };


        let session_res = self.repos.sessions_repo.create(&session).await?;

        info!("Created session: {:?}", session_res);

        let _ = self.repos.tokens_repo.create(&hashed_refresh_token, &TokenSession {
            user_id: session_res.user_id.to_string(),
            session_id: session_res.id.to_string(),
            device_info,
            expires_in: user_data.days_to_inactive,
            expires_at: session_res.expires_at,
            updated_at: session_res.updated_at,
            created_at: session_res.created_at,
        }).await?;

        let access_token = TokenBuilder::encode_access_token(String::from(user_data.user_id),
                                                             String::from(session_res.id),
                                                             app_state.get_secret_key())?;

        Ok((access_token, refresh_token))
    }
}

impl AuthUtilsService {
    pub fn new(repos: Arc<AuthRepositories>) -> Self {
        Self { repos }
    }
}