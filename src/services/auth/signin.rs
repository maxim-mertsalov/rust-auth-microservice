use std::sync::Arc;
use crate::dto::auth::signin::{FinalizeSignInReq, FinalizeSignInRes, InitEmailPassReq, InitEmailPassRes, InitEmailReq, InitEmailRes, ResendCodeReq, ResendCodeRes, SetPasswordReq, SetPasswordRes, VerifyEmailReq, VerifyEmailRes};
use crate::errors::app_error::AppError;
use crate::models::auth::signin::{SignInData, SignInSession, SignInState};
use crate::models::auth::utils::{Scope, TokenCreatorParams};
use crate::repositories::auth::{AuthRepositories};
use crate::repositories::auth::signin::SignInRepository;
use crate::repositories::auth::users::UserRepository;
use crate::services::auth::utils::{AuthUtilsService, IAuthUtilsService};
use crate::services::email_sender::EmailSender;
use crate::state::app_state::AppState;
use crate::utils::scope_validator::validate_scopes;

/// This trait defines all services related to user sign up, e.g. email confirmation, email and password validation, etc.
#[async_trait::async_trait]
pub trait ISignInService {
    async fn init_with_email_pass(&self, app_state: &AppState, user_req: InitEmailPassReq) -> Result<InitEmailPassRes, AppError>;
    async fn init_with_email(&self, app_state: &AppState, user_req: InitEmailReq) -> Result<InitEmailRes, AppError>;
    async fn set_password(&self, app_state: &AppState, user_req: SetPasswordReq) -> Result<SetPasswordRes, AppError>;
    async fn resend_code(&self, app_state: &AppState, user_req: ResendCodeReq) -> Result<ResendCodeRes, AppError>;
    async fn verify_email(&self, app_state: &AppState, user_req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError>;
    async fn finalise_session(&self, app_state: &AppState, user_req: FinalizeSignInReq) -> Result<FinalizeSignInRes, AppError>;
}

pub struct SignInService {
    repos: Arc<AuthRepositories>,
    token_creator: Arc<AuthUtilsService>,
    email_sender: Arc<dyn EmailSender + Send + Sync>,
}

#[async_trait::async_trait]
impl ISignInService for SignInService {
    async fn init_with_email_pass(&self, app_state: &AppState, user_req: InitEmailPassReq) -> Result<InitEmailPassRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::InitWithEmailPassStage;
        const NEXT_STAGE_2FA: SignInState = SignInState::InitWithEmailPassStage;
        const NEXT_STAGE_NO_2FA: SignInState = SignInState::Redirect;

        // Verify email
        let user = self.repos.users_repo.get_full_by_email(&user_req.email).await?
            .ok_or_else(|| AppError::BadRequest("Invalid email or password".to_string()))?;

        // Verify password
        if !bcrypt::verify(&user_req.password, &user.password)? {
            return Err(AppError::BadRequest("Invalid email or password".to_string()));
        }

        // Next stage
        let next_stage = if user.is_two_factor {
            NEXT_STAGE_2FA
        } else {
            NEXT_STAGE_NO_2FA
        };

        let session_id = uuid::Uuid::new_v4().to_string();

        let parsed_scopes = validate_scopes(&user_req.scopes);
        let mut session_data = SignInSession {
            scopes: parsed_scopes,
            device_info: user_req.device_info,
            final_redirect_url: user_req.final_redirect_url,
            stage: next_stage.clone(),
            data: SignInData {
                user_id: Some(String::from(user.id)),
                is_two_fa_enabled: user.is_two_factor,
                last_resent_code_at: None,
            },

            ..Default::default()
        };

        // Generate 2FA code if needed
        if user.is_two_factor {
            let otp_code = format!("{:06}", rand::random::<u32>() % 1_000_000);

            // Send OTP code to user's email
            self.email_sender.send_two_factor_code(&user.email, &otp_code).await;

            session_data.verification_code = otp_code;
            session_data.attempts = 0;
        }

        let _ = self.repos.signin_repo.create(&session_id, &session_data).await?;

        let response = InitEmailPassRes {
            session_token: session_id,
            next_stage,
        };

        Ok(response)
    }

    async fn init_with_email(&self, app_state: &AppState, user_req: InitEmailReq) -> Result<InitEmailRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::InitWithEmailStage;
        const NEXT_STAGE: SignInState = SignInState::SetPasswordStage;

        // Verify email
        let user = self.repos.users_repo.get_full_by_email(&user_req.email).await?
            .ok_or_else(|| AppError::BadRequest("No such user".to_string()))?;

        let session_id = uuid::Uuid::new_v4().to_string();

        let parsed_scopes = validate_scopes(&user_req.scopes);
        let session_data = SignInSession {
            scopes: parsed_scopes,
            device_info: user_req.device_info,
            final_redirect_url: user_req.final_redirect_url,
            stage: NEXT_STAGE,
            data: SignInData {
                user_id: Some(String::from(user.id)),
                is_two_fa_enabled: user.is_two_factor,
                last_resent_code_at: None,
            },

            ..Default::default()
        };

        let _ = self.repos.signin_repo.create(&session_id, &session_data).await?;

        let response = InitEmailRes {
            session_token: session_id,
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn set_password(&self, app_state: &AppState, user_req: SetPasswordReq) -> Result<SetPasswordRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::SetPasswordStage;
        const NEXT_STAGE_2FA: SignInState = SignInState::EmailVerificationStage;
        const NEXT_STAGE_NO_2FA: SignInState = SignInState::Redirect;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        let user = self.repos.users_repo.get_full_by_id(
            &session_data.data.user_id.as_ref()
                .ok_or_else(|| AppError::InternalServerError("User ID is not set in session".to_string()))?
        ).await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

        // Verify password
        if !bcrypt::verify(&user_req.password, &user.password)? {
            if session_data.attempts >= MAX_ATTEMPTS - 1 {
                self.repos.signin_repo.delete(&user_req.session_token).await?;

                return Err(AppError::BadRequest("Too many attempts. Try later".to_string()));
            }

            self.repos.signin_repo.increment_attempts(&user_req.session_token).await?;

            return Err(AppError::BadRequest("Invalid password".to_string()));
        }

        let next_stage = match session_data.data.is_two_fa_enabled {
            true => {
                let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

                session_data.verification_code = otp.clone();
                session_data.attempts = 0;
                session_data.data.last_resent_code_at = Some(chrono::Utc::now());

                self.email_sender.send_two_factor_code(&user.email, &otp).await;

                NEXT_STAGE_2FA
            }
            false => NEXT_STAGE_NO_2FA,
        };

        session_data.stage = next_stage.clone();
        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = SetPasswordRes { next_stage };

        Ok(response)
    }

    async fn resend_code(&self, app_state: &AppState, user_req: ResendCodeReq) -> Result<ResendCodeRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::EmailVerificationStage;
        const TIME_DIFFERENCE: u8 = 60; // seconds

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        let is_two_fa_enabled = session_data.data.is_two_fa_enabled;

        if !is_two_fa_enabled {
            return Err(AppError::BadRequest("Two-factor authentication is not enabled for this user".to_string()));
        }

        let last_resent = session_data.data.last_resent_code_at
            .ok_or_else(|| AppError::BadRequest("Last resent code time is not set".to_string()))?;

        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last_resent);

        if duration.num_seconds() < TIME_DIFFERENCE as i64 {
            return Err(AppError::BadRequest(format!("You can resend the code after {} seconds", TIME_DIFFERENCE)));
        }

        let user = self.repos.users_repo.get_full_by_id(
            &session_data.data.user_id.as_ref()
                .ok_or_else(|| AppError::InternalServerError("User ID is not set in session".to_string()))?
        ).await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

        let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

        self.email_sender.send_two_factor_code(&user.email, &otp).await;

        session_data.verification_code = otp;
        session_data.attempts = 0;

        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = ResendCodeRes;

        Ok(response)
    }

    async fn verify_email(&self, app_state: &AppState, user_req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::EmailVerificationStage;
        const NEXT_STAGE: SignInState = SignInState::Redirect;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if session_data.verification_code != user_req.verification_code {
            if session_data.attempts >= MAX_ATTEMPTS - 1 {
                self.repos.signin_repo.delete(&user_req.session_token).await?;

                return Err(AppError::BadRequest("Too many attempts. Try later".to_string()));
            }

            let _ = self.repos.signin_repo.increment_attempts(&user_req.session_token).await?;

            return Err(AppError::BadRequest("Invalid verification code".to_string()));
        }

        session_data.stage = NEXT_STAGE;
        let _ = self.repos.signin_repo.update_stage(&user_req.session_token, NEXT_STAGE).await?;

        let response = VerifyEmailRes {
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn finalise_session(&self, app_state: &AppState, user_req: FinalizeSignInReq) -> Result<FinalizeSignInRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::Redirect;

        let Some(session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        // Delete sign-in session
        let _ = self.repos.signin_repo.delete(&user_req.session_token).await?;

        let user_id = session_data.data.user_id.as_ref()
            .ok_or_else(|| AppError::BadRequest("User is not found in session".to_string()))?;

        // Fetch user
        let user = self.repos.users_repo.get_full_by_id(user_id).await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

        // Create tokens and session
        let token_params = TokenCreatorParams {
            user_id: user.id,
            days_to_inactive: user.days_to_inactive,
        };

        let (access_token, refresh_token) = self.token_creator
            .create_tokens_and_session(app_state, &token_params, session_data.device_info).await?;

        let mut response = FinalizeSignInRes {
            redirect_url: session_data.final_redirect_url,
            ..Default::default()
        };

        for scope in &session_data.scopes {
            match scope {
                Scope::OpenId => {
                    response.access_token = Some(access_token.clone());
                }
                Scope::Profile => {
                    response.first_name = Some(user.first_name.clone());
                    response.last_name = Some(user.last_name.clone());
                }
                Scope::Email => {
                    response.email = Some(user.email.clone());
                }
                Scope::OfflineAccess => {
                    response.refresh_token = Some(refresh_token.clone());
                }
            }
        }

        Ok(response)
    }
}

impl SignInService {
    pub fn new(repos: Arc<AuthRepositories>, token_creator: Arc<AuthUtilsService>, email_sender: Arc<dyn EmailSender + Send + Sync>) -> Self {
        Self { repos, token_creator, email_sender }
    }
}