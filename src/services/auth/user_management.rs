use std::sync::Arc;
use crate::dto::auth::user_management::{DeleteUserReq, UpdateUserReq};
use crate::errors::app_error::AppError;
use crate::models::auth::users::{MidFieldsUser, MinFieldsUser};
use crate::repositories::auth::AuthRepositories;
use crate::repositories::auth::sessions::SessionRepository;
use crate::repositories::auth::tokens::TokenSessionRepository;
use crate::repositories::auth::users::UserRepository;
use crate::state::app_state::AppState;
use crate::utils::access_tokens::TokenBuilder;


#[async_trait::async_trait]
pub trait IUserManagementService {
    async fn get_users(&self) -> Result<Vec<MinFieldsUser>, AppError>;
    async fn get_user(&self, user_id: &str) -> Result<Option<MidFieldsUser>, AppError>;
    async fn update_user(&self, app_state: &AppState, user_req: &UpdateUserReq) -> Result<MidFieldsUser, AppError>;
    async fn delete_user(&self, app_state: &AppState, req: DeleteUserReq) -> Result<(), AppError>;
    // async fn reset_password(&self, app_state: &AppState) -> Result<(), AppError>;
}

pub struct UserManagementService{
    repos: Arc<AuthRepositories>,
}

#[async_trait::async_trait]
impl IUserManagementService for UserManagementService{
    /// ----- Get all users -----
    async fn get_users(&self) -> Result<Vec<MinFieldsUser>, AppError> {
        self.repos.users_repo.get_all().await.map_err(AppError::from)
    }

    async fn get_user(&self, user_id: &str) -> Result<Option<MidFieldsUser>, AppError> {
        self.repos.users_repo.get_one(user_id).await.map_err(AppError::from)
    }

    async fn update_user(&self, app_state: &AppState, user_req: &UpdateUserReq) -> Result<MidFieldsUser, AppError> {
        let (access_token_data, exp) = TokenBuilder::decode_access_token(&user_req.access_token, app_state.get_secret_key())?;

        if exp {
            return Err(AppError::ExpiredAccessToken);
        }

        if user_req.first_name.is_none() && user_req.last_name.is_none() {
            return Err(AppError::BadRequest("At least one field must be provided for update".to_string()));
        }

        self.repos.users_repo.update_one(&access_token_data.sub, user_req).await.map_err(AppError::from)
    }

    /// ----- Delete user by ID, along with all sessions -----
    async fn delete_user(&self, app_state: &AppState, req: DeleteUserReq) -> Result<(), AppError> {
        let access_token_data = TokenBuilder::decode_access_token(&req.access_token, app_state.get_secret_key())?;

        // Expired token
        if access_token_data.1 {
            return Err(AppError::ExpiredAccessToken);
        }

        let sessions = self.repos.sessions_repo.get_all_by_user_id(&access_token_data.0.sub).await?;

        for session in sessions {
            let _ = self.repos.tokens_repo.delete(&session.refresh_token).await;
        }

        self.repos.users_repo.delete_by_id(&access_token_data.0.sub).await.map_err(AppError::from)
    }
}

impl UserManagementService {
    pub fn new(repos: Arc<AuthRepositories>) -> Self {
        Self { repos }
    }
}