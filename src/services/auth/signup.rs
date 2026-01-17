use std::sync::Arc;
use log::info;
use crate::dto::auth::signup::{FinalizeSignUpReq, FinalizeSignUpRes, InitProfileReq, InitProfileRes, InitSessionReq, InitSessionRes, ResendEmailCodeReq, ResendEmailCodeRes, ReturnBackSessionReq, ReturnBackSessionRes, SetEmailReq, SetEmailRes, SetPasswordReq, SetPasswordRes, SetProfileReq, SetProfileRes, VerifyEmailReq, VerifyEmailRes};
use crate::errors::app_error::AppError;
use crate::models::auth::signup::{SignUpData, SignUpSession, SignUpState};
use crate::models::auth::users::User;
use crate::models::auth::utils::{Scope, TokenCreatorParams};
use crate::repositories::auth::{AuthRepositories};
use crate::repositories::auth::signup::SignUpRepository;
use crate::repositories::auth::users::UserRepository;
use crate::services::auth::utils::{AuthUtilsService, IAuthUtilsService};
use crate::services::email_sender::EmailSender;
use crate::state::app_state::AppState;
use crate::utils::scope_validator::validate_scopes;

/// This trait defines all services related to user sign up, e.g. email confirmation, email and password validation, etc.
#[async_trait::async_trait]
pub trait ISignUpService {
    async fn init_session(&self, app_state: &AppState, user_req: InitSessionReq) -> Result<InitSessionRes, AppError>;
    async fn init_session_with_profile(&self, app_state: &AppState, user_req: InitProfileReq) -> Result<InitProfileRes, AppError>;
    async fn set_profile_session(&self, app_state: &AppState, user_req: SetProfileReq) -> Result<SetProfileRes, AppError>;
    async fn set_email_session(&self, app_state: &AppState, user_req: SetEmailReq) -> Result<SetEmailRes, AppError>;
    async fn resend_email_verification(&self, app_state: &AppState, user_req: ResendEmailCodeReq) -> Result<ResendEmailCodeRes, AppError>;
    async fn verify_email_session(&self, app_state: &AppState, user_req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError>;
    async fn set_password_session(&self, app_state: &AppState, user_req: SetPasswordReq) -> Result<SetPasswordRes, AppError>;
    async fn finalise_session(&self, app_state: &AppState, user_req: FinalizeSignUpReq) -> Result<FinalizeSignUpRes, AppError>;
    async fn return_back(&self, app_state: &AppState, user_req: ReturnBackSessionReq) -> Result<ReturnBackSessionRes, AppError>;
}

pub struct SignUpService {
    repos: Arc<AuthRepositories>,
    token_creator: Arc<AuthUtilsService>,
    email_sender: Arc<dyn EmailSender + Send + Sync>,
}

#[async_trait::async_trait]
impl ISignUpService for SignUpService {
    async fn init_session(&self, app_state: &AppState, user_req: InitSessionReq) -> Result<InitSessionRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::InitStage;
        const NEXT_STAGE: SignUpState = SignUpState::ProfileStage;

        // Get all data from request and create signup session
        let parsed_scopes = validate_scopes(&user_req.scopes);

        let session_id = uuid::Uuid::new_v4().to_string();

        let session_data = SignUpSession {
            scopes: parsed_scopes,
            final_redirect_url: user_req.final_redirect_url,
            device_info: user_req.device_info,
            stage: NEXT_STAGE,

            ..Default::default()
        };

        let _ = self.repos.signup_repo.create(&session_id, &session_data).await?;

        let response = InitSessionRes {
            session_token: session_id,
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn init_session_with_profile(&self, app_state: &AppState, user_req: InitProfileReq) -> Result<InitProfileRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::InitWithProfileStage;
        const NEXT_STAGE: SignUpState = SignUpState::EmailStage;

        // Get all data from request and create signup session
        let parsed_scopes = validate_scopes(&user_req.scopes);

        let session_id = uuid::Uuid::new_v4().to_string();

        let session_data = SignUpSession {
            scopes: parsed_scopes,
            device_info: user_req.device_info,
            final_redirect_url: user_req.final_redirect_url,

            data: SignUpData {
                first_name: Some(user_req.first_name),
                last_name: Some(user_req.last_name),
                ..Default::default()
            },

            stage: NEXT_STAGE,

            ..Default::default()
        };

        let _ = self.repos.signup_repo.create(&session_id, &session_data).await?;

        let response = InitProfileRes {
            session_token: session_id,
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn set_profile_session(&self, app_state: &AppState, user_req: SetProfileReq) -> Result<SetProfileRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::ProfileStage;
        const NEXT_STAGE: SignUpState = SignUpState::EmailStage;

        // check session stage
        let Some(mut session) = self.repos.signup_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-up session not found".to_string()))
        };

        if session.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid signup session stage".to_string()));
        }

        session.data.first_name = Some(user_req.first_name);
        session.data.last_name = Some(user_req.last_name);

        // Change stage
        session.stage = NEXT_STAGE;

        let _ = self.repos.signup_repo.update(&user_req.session_token, &session).await?;

        let response = SetProfileRes {
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn set_email_session(&self, app_state: &AppState, user_req: SetEmailReq) -> Result<SetEmailRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::EmailStage;
        const NEXT_STAGE: SignUpState = SignUpState::EmailVerificationStage; // you can skip email verification and set password stage directly

        // check session stage
        let Some(mut session) = self.repos.signup_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-up session not found".to_string()))
        };

        if session.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid signup session stage".to_string()));
        }

        // verify email uniqueness
        let email_exists = self.repos.users_repo.get_full_by_email(&user_req.email).await?.is_some();
        if email_exists {
            return Err(AppError::BadRequest("Email is already in use".to_string()));
        }

        // Send verification code to user's email
        let verification_code = format!("{:06}", rand::random::<u32>() % 1_000_000);
        self.email_sender.send_confirmation_sign_up(&user_req.email, &verification_code).await;

        session.data.email = Some(user_req.email);
        session.verification_code = verification_code;
        session.attempts = 0;
        session.data.last_resent_code_at = Some(chrono::Utc::now());

        // Change stage
        session.stage = NEXT_STAGE;

        let _ = self.repos.signup_repo.update(&user_req.session_token, &session).await?;

        let response = SetEmailRes {
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn resend_email_verification(&self, app_state: &AppState, user_req: ResendEmailCodeReq) -> Result<ResendEmailCodeRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::EmailVerificationStage;
        const TIME_DIFFERENCE: u8 = 60; // seconds

        // check session stage
        let Some(mut session) = self.repos.signup_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-up session not found".to_string()))
        };

        if session.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid signup session stage".to_string()));
        }

        let last_resent = session.data.last_resent_code_at
            .ok_or_else(|| AppError::BadRequest("Last resent code time is not set".to_string()))?;

        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last_resent);

        if duration.num_seconds() < TIME_DIFFERENCE as i64 {
            return Err(AppError::BadRequest(format!("You can resend the code after {} seconds", TIME_DIFFERENCE)));
        }

        let email = session.data.email.as_ref()
            .ok_or_else(|| AppError::InternalServerError("Missing email address".to_string()))?;

        let verification_code = format!("{:06}", rand::random::<u32>() % 1_000_000);
        self.email_sender.send_confirmation_sign_up(email, &verification_code).await;

        session.verification_code = verification_code;
        session.attempts = 0;
        session.data.last_resent_code_at = Some(chrono::Utc::now());

        let _ = self.repos.signup_repo.update(&user_req.session_token, &session).await?;

        let response = ResendEmailCodeRes;

        Ok(response)
    }

    async fn verify_email_session(&self, app_state: &AppState, user_req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::EmailVerificationStage;
        const NEXT_STAGE: SignUpState = SignUpState::PasswordStage;
        const MAX_ATTEMPTS: u8 = 5;

        // check session stage
        let Some(mut session) = self.repos.signup_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-up session not found".to_string()))
        };

        if session.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid signup session stage".to_string()));
        }

        // Verify code
        if user_req.verification_code != session.verification_code {
            if session.attempts >= MAX_ATTEMPTS - 1 {
                return Err(AppError::BadRequest("Too many attempts. Make sure you've entered the correct email".to_string()));
            }

            let _ = self.repos.signup_repo.increment_attempts(&user_req.session_token).await?;
            return Err(AppError::BadRequest("Invalid verification code".to_string()));
        }


        // Change stage
        session.stage = NEXT_STAGE;

        let _ = self.repos.signup_repo.update(&user_req.session_token, &session).await?;

        let response = VerifyEmailRes {
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn set_password_session(&self, app_state: &AppState, user_req: SetPasswordReq) -> Result<SetPasswordRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::PasswordStage;
        const NEXT_STAGE: SignUpState = SignUpState::Redirect;

        // check session stage
        let Some(mut session) = self.repos.signup_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-up session not found".to_string()))
        };

        if session.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid signup session stage".to_string()));
        }

        let hashed_password = bcrypt::hash(&user_req.password, bcrypt::DEFAULT_COST)
            .map_err(|_| AppError::InternalServerError("Failed to hash password".to_string()))?;

        session.data.password = Some(hashed_password);

        // Change stage
        session.stage = NEXT_STAGE;

        let _ = self.repos.signup_repo.update(&user_req.session_token, &session).await?;

        let response = SetPasswordRes {
            next_stage: NEXT_STAGE,
        };

        Ok(response)
    }

    async fn finalise_session(&self, app_state: &AppState, user_req: FinalizeSignUpReq) -> Result<FinalizeSignUpRes, AppError> {
        const CURRENT_STAGE: SignUpState = SignUpState::Redirect;

        // check session stage
        let Some(mut session) = self.repos.signup_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-up session not found".to_string()))
        };

        if session.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid signup session stage".to_string()));
        }

        let _ = self.repos.signup_repo.delete(&user_req.session_token).await?;

        let user = User {
            id: uuid::Uuid::now_v7(),
            email: session.data.email.clone().ok_or(AppError::BadRequest("Email not set in signup session".to_string()))?,
            first_name: session.data.first_name.clone().unwrap_or_default(),
            last_name: session.data.last_name.clone().unwrap_or_default(),
            is_two_factor: false,
            is_verified: true,
            password: session.data.password.clone().ok_or(AppError::BadRequest("Password not set in signup session".to_string()))?,
            updated_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            days_to_inactive: 30,
        };
        let user_res = self.repos.users_repo.create(&user).await?;

        let token_params = TokenCreatorParams {
            user_id: user_res.id,
            days_to_inactive: user.days_to_inactive,
        };

        let (access_token, refresh_token) = self.token_creator.create_tokens_and_session(app_state, &token_params, session.device_info.clone()).await?;

        let mut response = FinalizeSignUpRes {
            redirect_url: session.final_redirect_url,

            ..Default::default()
        };


        for scope in &session.scopes {
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

    async fn return_back(&self, app_state: &AppState, user_req: ReturnBackSessionReq) -> Result<ReturnBackSessionRes, AppError> {

        // check session stage
        let Some(mut session) = self.repos.signup_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-up session not found".to_string()))
        };

        match session.stage {
            // SignUpState::InitStage => {}
            // SignUpState::InitWithProfileStage => {}
            // SignUpState::ProfileStage => {}
            SignUpState::EmailStage => {
                session.stage = SignUpState::ProfileStage;
            }
            SignUpState::EmailVerificationStage => {
                session.stage = SignUpState::EmailStage;
            }
            SignUpState::PasswordStage => {
                session.stage = SignUpState::EmailVerificationStage;
            }
            SignUpState::Redirect => {
                session.stage = SignUpState::PasswordStage;
            }
            _ => {
                return Err(AppError::BadRequest("You can't return back from this stage".to_string()));
            }
        }

        let _ = self.repos.signup_repo.update(&user_req.session_token, &session).await?;
        let response = ReturnBackSessionRes {
            next_stage: session.stage,
        };
        Ok(response)
    }
}

impl SignUpService {
    pub fn new(repos: Arc<AuthRepositories>, token_creator: Arc<AuthUtilsService>, email_sender: Arc<dyn EmailSender + Send + Sync>) -> Self {
        Self { repos, token_creator, email_sender }
    }


}