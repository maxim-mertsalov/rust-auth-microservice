use std::sync::Arc;
use crate::dto::auth::security_management::{AddRecoveryEmailReq, AddRecoveryEmailRes, ChangeEmailReq, ChangeEmailRes, ChangePasswordReq, ChangePasswordRes, ChangeTwoFactorStatusReq, ChangeTwoFactorStatusRes, GenerateRecoveryCodesReq, GenerateRecoveryCodesRes, RemoveRecoveryEmailReq, RemoveRecoveryEmailRes, VerifyEmailReq, VerifyEmailRes, VerifyRecoveryEmailReq, VerifyRecoveryEmailRes};
use crate::errors::app_error::AppError;
use crate::repositories::auth::AuthRepositories;
use crate::repositories::auth::recovery_codes::RecoveryCodesRepository;
use crate::repositories::auth::users::UserRepository;
use crate::services::auth::utils::AuthUtilsService;
use crate::services::email_sender::EmailSender;
use crate::state::app_state::AppState;
use crate::utils::access_tokens::TokenBuilder;
use crate::utils::string::StringUtils;
use crate::utils::sudo_tokens::SudoTokenBuilder;

#[async_trait::async_trait]
pub trait ISecurityManagementService {
    async fn change_password(&self, app_state: &AppState, req: ChangePasswordReq) -> Result<ChangePasswordRes, AppError>;

    async fn generate_new_recovery_codes(&self, app_state: &AppState, req: GenerateRecoveryCodesReq) -> Result<GenerateRecoveryCodesRes, AppError>;

    async fn change_email(&self, app_state: &AppState, req: ChangeEmailReq) -> Result<ChangeEmailRes, AppError>;
    async fn verify_email_change(&self, app_state: &AppState, req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError>;

    async fn change_2fa_method(&self, app_state: &AppState, req: ChangeTwoFactorStatusReq) -> Result<ChangeTwoFactorStatusRes, AppError>;

    async fn add_recovery_email(&self, app_state: &AppState, req: AddRecoveryEmailReq) -> Result<AddRecoveryEmailRes, AppError>;
    async fn verify_recovery_email(&self, app_state: &AppState, req: VerifyRecoveryEmailReq) -> Result<VerifyRecoveryEmailRes, AppError>;
    async fn remove_recovery_email(&self, app_state: &AppState, req: RemoveRecoveryEmailReq) -> Result<RemoveRecoveryEmailRes, AppError>;
}

pub struct SecurityManagementService {
    repos: Arc<AuthRepositories>,
    token_creator: Arc<AuthUtilsService>,
    email_sender: Arc<dyn EmailSender + Send + Sync>,
}

#[async_trait::async_trait]
impl ISecurityManagementService for SecurityManagementService {
    async fn change_password(&self, app_state: &AppState, req: ChangePasswordReq) -> Result<ChangePasswordRes, AppError> {
        let user_id = self.check_both_tokens(app_state, &req.access_token, &req.sudo_token).await?;

        self.repos.users_repo.update_password(&user_id, &req.new_password).await?;

        Ok(ChangePasswordRes)
    }

    async fn generate_new_recovery_codes(&self, app_state: &AppState, req: GenerateRecoveryCodesReq) -> Result<GenerateRecoveryCodesRes, AppError> {
        const NUM_CODES: usize = 10;
        const CODE_LENGTH: usize = 16;
        const CODE_PARTS: usize = 4;

        let user_id = self.check_both_tokens(app_state, &req.access_token, &req.sudo_token).await?;

        let mut recovery_codes: Vec<(String, String)> = Vec::new();

        for _ in 0..NUM_CODES {
            let full_code = StringUtils::generate_random_string(CODE_LENGTH);
            let prefix = full_code.chars().take(4).collect::<String>();
            let code = full_code.chars().skip(4).collect::<String>();

            recovery_codes.push((prefix, code));
        }

        let res = self.repos.recovery_codes_repo.create(&user_id, recovery_codes).await?;

        let mut formated_codes: Vec<String> = Vec::with_capacity(res.len());

        for code in &res {
            let full_code = format!("{}{}", code.prefix, code.recovery_code);
            let formated_code = StringUtils::add_delimiter(&full_code, CODE_PARTS);
            formated_codes.push(formated_code);
        }

        let response = GenerateRecoveryCodesRes {
            recovery_codes: formated_codes,
        };

        Ok(response)
    }

    async fn change_email(&self, app_state: &AppState, req: ChangeEmailReq) -> Result<ChangeEmailRes, AppError> {
        todo!()
    }

    async fn verify_email_change(&self, app_state: &AppState, req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError> {
        todo!()
    }

    async fn change_2fa_method(&self, app_state: &AppState, req: ChangeTwoFactorStatusReq) -> Result<ChangeTwoFactorStatusRes, AppError> {
        todo!()
    }

    async fn add_recovery_email(&self, app_state: &AppState, req: AddRecoveryEmailReq) -> Result<AddRecoveryEmailRes, AppError> {
        todo!()
    }

    async fn verify_recovery_email(&self, app_state: &AppState, req: VerifyRecoveryEmailReq) -> Result<VerifyRecoveryEmailRes, AppError> {
        todo!()
    }

    async fn remove_recovery_email(&self, app_state: &AppState, req: RemoveRecoveryEmailReq) -> Result<RemoveRecoveryEmailRes, AppError> {
        todo!()
    }
}

impl SecurityManagementService {
    pub fn new(repos: Arc<AuthRepositories>, token_creator: Arc<AuthUtilsService>, email_sender: Arc<dyn EmailSender + Send + Sync>) -> Self {
        Self {
            repos,
            token_creator,
            email_sender,
        }
    }

    async fn check_access_token(&self, app_state: &AppState, access_token: &str) -> Result<String, AppError> {
        let (token_data, exp) = TokenBuilder::decode_access_token(access_token, app_state.get_secret_key())?;

        if exp {
            return Err(AppError::ExpiredAccessToken);
        }

        Ok(token_data.sub)
    }

    async fn check_sudo_token(&self, app_state: &AppState, sudo_token: &str) -> Result<String, AppError> {
        let (token_data, exp) = SudoTokenBuilder::decode_sudo_token(sudo_token, app_state.get_sudo_secret_key())?;

        if exp {
            return Err(AppError::BadRequest("Sudo token is expired".to_string()));
        }

        Ok(token_data.sub)
    }

    async fn check_both_tokens(&self, app_state: &AppState, access_token: &str, sudo_token: &str) -> Result<String, AppError> {
        let user_id_from_access = self.check_access_token(app_state, access_token).await?;
        let user_id_from_sudo = self.check_sudo_token(app_state, sudo_token).await?;

        if user_id_from_access != user_id_from_sudo {
            return Err(AppError::Unauthorized("Access token and sudo token do not belong to the same user".to_string()));
        }

        Ok(user_id_from_access)
    }
}