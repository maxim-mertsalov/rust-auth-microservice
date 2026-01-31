use std::sync::Arc;
use crate::dto::auth::signin::{FinalizeSignInReq, FinalizeSignInRes, InitEmailReq, InitEmailRes, ResendCodeReq, ResendCodeRes, SetPasswordReq, SetPasswordRes, VerifyEmailReq, VerifyEmailRes};
use crate::dto::auth::utils::TokenCreatorParams;
use crate::errors::app_error::AppError;
use crate::models::auth::signin::{SignInFlow, SignInFlowIdentifier, SignInSession, SignInState};
use crate::models::auth::utils::{FlowMetadata, Scope};
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
    async fn init_with_email(&self, app_state: &AppState, user_req: InitEmailReq) -> Result<InitEmailRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::InitWithEmailStage;
        const NEXT_STAGE: SignInState = SignInState::WithPasswordStage;

        // Verify email
        let user = self.repos.users_repo.get_full_by_email(&user_req.email).await?
            .ok_or_else(|| AppError::BadRequest("No such user".to_string()))?;

        let session_id = uuid::Uuid::new_v4().to_string();

        let parsed_scopes = validate_scopes(&user_req.scopes);
        let session_data = SignInSession {
            metadata: FlowMetadata {
                scopes: parsed_scopes,
                final_redirect_url: user_req.final_redirect_url,
                device_info: user_req.device_info,
            },
            identifier: SignInFlowIdentifier {
                user_id: user.id.to_string(),
                mfa_enabled: user.is_two_factor,
            },
            flow: SignInFlow {
                satisfied_methods: vec![], //TODO: not implemented yet
                available_methods: vec![], //TODO: not implemented yet
                current_method: None,
                stage: NEXT_STAGE,
            },
            data: Default::default(),
        };

        let _ = self.repos.signin_repo.create(&session_id, &session_data).await?;

        let response = InitEmailRes {
            session_token: session_id,
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn set_password(&self, app_state: &AppState, user_req: SetPasswordReq) -> Result<SetPasswordRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::WithPasswordStage;
        const NEXT_STAGE_2FA: SignInState = SignInState::VerifyEmailStage;
        const NEXT_STAGE_NO_2FA: SignInState = SignInState::Redirect;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        let user = self.repos.users_repo.get_full_by_id(
            &session_data.identifier.user_id
        ).await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

        // Verify password
        if !bcrypt::verify(&user_req.password, &user.password)? {
            if session_data.data.attempts >= MAX_ATTEMPTS - 1 {
                self.repos.signin_repo.delete(&user_req.session_token).await?;

                return Err(AppError::BadRequest("Too many attempts. Try later".to_string()));
            }

            session_data.data.attempts += 1;

            self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

            return Err(AppError::BadRequest("Invalid password".to_string()));
        }

        let next_stage = match session_data.identifier.mfa_enabled {
            true => {
                let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

                self.email_sender.send_two_factor_code(&user.email, &otp).await;

                session_data.data.code = Some(otp);
                session_data.data.attempts = 0;
                session_data.data.last_resent_code_at = Some(chrono::Utc::now());

                NEXT_STAGE_2FA
            }
            false => NEXT_STAGE_NO_2FA,
        };

        session_data.flow.stage = next_stage.clone();
        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = SetPasswordRes { next_stage };

        Ok(response)
    }

    async fn resend_code(&self, app_state: &AppState, user_req: ResendCodeReq) -> Result<ResendCodeRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::VerifyEmailStage;
        const TIME_DIFFERENCE: u8 = 60; // seconds

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        let is_two_fa_enabled = session_data.identifier.mfa_enabled;

        if !is_two_fa_enabled {
            return Err(AppError::BadRequest("Nothing to resend".to_string()));
        }

        let last_resent = session_data.data.last_resent_code_at
            .ok_or_else(|| AppError::BadRequest("Last resent code time is not set".to_string()))?;

        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last_resent);

        if duration.num_seconds() < TIME_DIFFERENCE as i64 {
            return Err(AppError::BadRequest(format!("You can resend the code after {} seconds", TIME_DIFFERENCE)));
        }

        let user = self.repos.users_repo.get_full_by_id(
            &session_data.identifier.user_id
        ).await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

        let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

        self.email_sender.send_two_factor_code(&user.email, &otp).await;

        session_data.data.code = Some(otp);
        session_data.data.attempts = 0;
        session_data.data.last_resent_code_at = Some(chrono::Utc::now());

        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = ResendCodeRes;

        Ok(response)
    }

    async fn verify_email(&self, app_state: &AppState, user_req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::VerifyEmailStage;
        const NEXT_STAGE: SignInState = SignInState::Redirect;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        let code = session_data.data.code.as_ref().ok_or_else(||
            AppError::InternalServerError("Verification code is not set".to_string()) )?;

        if *code != user_req.verification_code {
            if session_data.data.attempts >= MAX_ATTEMPTS - 1 {
                // self.repos.signin_repo.delete(&user_req.session_token).await?;
                // TODO: too harsh? I think we should regenerate code and also add global regeneration limit

                //TODO: Add ip to blacklist for this user if regeneration limit exceeded

                return Err(AppError::BadRequest("Too many attempts. Try later".to_string()));
            }

            session_data.data.attempts += 1;

            let _ = self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

            return Err(AppError::BadRequest("Invalid verification code".to_string()));
        }

        session_data.flow.stage = NEXT_STAGE;
        session_data.data.code = None;
        session_data.data.last_resent_code_at = None;
        session_data.data.attempts = 0;

        let _ = self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

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

        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        // Delete sign-in session
        let _ = self.repos.signin_repo.delete(&user_req.session_token).await?;

        let user_id = session_data.identifier.user_id;

        // Fetch user
        let user = self.repos.users_repo.get_full_by_id(&user_id).await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

        // Create tokens and session
        let token_params = TokenCreatorParams {
            user_id: user.id,
            days_to_inactive: user.days_to_inactive,
        };

        let (access_token, refresh_token) = self.token_creator
            .create_tokens_and_session(app_state, &token_params, session_data.metadata.device_info).await?;

        let mut response = FinalizeSignInRes {
            redirect_url: session_data.metadata.final_redirect_url,
            ..Default::default()
        };

        for scope in &session_data.metadata.scopes {
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