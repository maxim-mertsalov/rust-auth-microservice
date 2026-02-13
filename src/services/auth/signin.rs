use std::sync::Arc;
use crate::dto::auth::signin::{FinalizeSignInReq, FinalizeSignInRes, GetAuthMethodsReq, GetAuthMethodsRes, GetRecoveryEmailsReq, GetRecoveryEmailsRes, InitEmailReq, InitEmailRes, ResendCodeReq, ResendCodeRes, SelectAuthMethodReq, SelectAuthMethodRes, SelectRecoveryEmailReq, SelectRecoveryEmailRes, SetPasswordReq, SetPasswordRes, SetRecoveryCodeReq, SetRecoveryCodeRes, VerifyEmailReq, VerifyEmailRes, VerifyRecoveryEmailCodeReq, VerifyRecoveryEmailCodeRes};
use crate::dto::auth::utils::TokenCreatorParams;
use crate::errors::app_error::AppError;
use crate::models::auth::signin::{AuthenticationMethod, SignInFlow, SignInFlowIdentifier, SignInSession, SignInState};
use crate::models::auth::users::User;
use crate::models::auth::utils::{FlowMetadata, Scope};
use crate::repositories::auth::{AuthRepositories};
use crate::repositories::auth::recovery_codes::RecoveryCodesRepository;
use crate::repositories::auth::recovery_emails::RecoveryEmailsRepository;
use crate::repositories::auth::signin::SignInRepository;
use crate::repositories::auth::users::UserRepository;
use crate::services::auth::utils::{AuthUtilsService, IAuthUtilsService};
use crate::services::email_sender::EmailSender;
use crate::state::app_state::AppState;
use crate::utils::scope_validator::validate_scopes;
use crate::utils::signin_utils::mask_emails;

/// This trait defines all services related to user sign up, e.g. email confirmation, email and password validation, etc.
#[async_trait::async_trait]
pub trait ISignInService {
    // I. Initialize sign-in methods
    async fn init_with_email(&self, app_state: &AppState, user_req: InitEmailReq) -> Result<InitEmailRes, AppError>;

    // II. Authentication methods
    // 0. Select authentication method (if multiple available)
    async fn get_auth_methods(&self, app_state: &AppState, user_req: GetAuthMethodsReq) -> Result<GetAuthMethodsRes, AppError>;
    async fn select_auth_method(&self, app_state: &AppState, user_req: SelectAuthMethodReq) -> Result<SelectAuthMethodRes, AppError>;


    // 1. password method
    async fn set_password(&self, app_state: &AppState, user_req: SetPasswordReq) -> Result<SetPasswordRes, AppError>;

    // 2. recovery email code method
    async fn get_recovery_emails(&self, app_state: &AppState, user_req: GetRecoveryEmailsReq) -> Result<GetRecoveryEmailsRes, AppError>;
    async fn select_recovery_email(&self, app_state: &AppState, user_req: SelectRecoveryEmailReq) -> Result<SelectRecoveryEmailRes, AppError>;
    async fn verify_recovery_email_code(&self, app_state: &AppState, user_req: VerifyRecoveryEmailCodeReq) -> Result<VerifyRecoveryEmailCodeRes, AppError>;

    // 3. recovery code method
    async fn set_recovery_code(&self, app_state: &AppState, user_req: SetRecoveryCodeReq) -> Result<SetRecoveryCodeRes, AppError>;


    // 4. email verification method
    async fn verify_email(&self, app_state: &AppState, user_req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError>;

    // 2 + 4. resend code for recovery email and email verification
    async fn resend_code(&self, app_state: &AppState, user_req: ResendCodeReq) -> Result<ResendCodeRes, AppError>;


    // III. Final
    async fn final_session(&self, app_state: &AppState, user_req: FinalizeSignInReq) -> Result<FinalizeSignInRes, AppError>;
}

pub struct SignInService {
    repos: Arc<AuthRepositories>,
    token_creator: Arc<AuthUtilsService>,
    email_sender: Arc<dyn EmailSender + Send + Sync>,
}

#[async_trait::async_trait]
impl ISignInService for SignInService {
    async fn init_with_email(&self, app_state: &AppState, user_req: InitEmailReq) -> Result<InitEmailRes, AppError> {
        const NEXT_STAGE: SignInState = SignInState::WithPasswordStage;

        // Verify email
        let user = self.repos.users_repo.get_full_by_email(&user_req.email).await?
            .ok_or_else(|| AppError::BadRequest("No such user".to_string()))?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let user_id = user.id.to_string();

        //TODO: check if flag allow_email_verification is enabled for this user
        let mut available_methods: Vec<AuthenticationMethod> = vec![AuthenticationMethod::Password, AuthenticationMethod::EmailVerification];
        if self.repos.recovery_emails_repo.exists_verified(&user_id).await? {
            available_methods.push(AuthenticationMethod::RecoveryEmailCode);
        }
        if self.repos.recovery_codes_repo.exists(&user_id).await? {
            available_methods.push(AuthenticationMethod::RecoveryCode);
        }

        let parsed_scopes = validate_scopes(&user_req.scopes);
        let session_data = SignInSession {
            metadata: FlowMetadata {
                scopes: parsed_scopes,
                final_redirect_url: user_req.final_redirect_url,
                device_info: user_req.device_info,
            },
            identifier: SignInFlowIdentifier {
                user_id,
                mfa_enabled: user.is_two_factor,
            },
            flow: SignInFlow {
                satisfied_methods: vec![],
                available_methods,
                blocked_methods: vec![],
                incorrect_attempts: 0,
                current_method: Some(AuthenticationMethod::Password),
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

    async fn get_auth_methods(&self, app_state: &AppState, user_req: GetAuthMethodsReq) -> Result<GetAuthMethodsRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::SelectAuthenticationMethodsStage;
        const AVAILABLE_STAGES: &[SignInState] = &[
            SignInState::SelectAuthenticationMethodsStage,
            SignInState::WithPasswordStage,
            SignInState::ChoseRecoveryEmailStage,
            SignInState::VerifyRecoveryEmailCodeStage,
            SignInState::WithRecoveryCodeStage,
            SignInState::VerifyEmailStage,
        ];

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if !AVAILABLE_STAGES.contains(&session_data.flow.stage) {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        self.check_global_max_attempts(&user_req.session_token, &session_data).await?;

        //* change stage, if needed
        if session_data.flow.stage != CURRENT_STAGE {
            session_data.flow.stage = CURRENT_STAGE;

            let _ = self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;
        }

        let response = GetAuthMethodsRes {
            available_methods: session_data.flow.available_methods,
        };

        Ok(response)
    }

    async fn select_auth_method(&self, app_state: &AppState, user_req: SelectAuthMethodReq) -> Result<SelectAuthMethodRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::SelectAuthenticationMethodsStage;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        self.check_global_max_attempts(&user_req.session_token, &session_data).await?;

        if !session_data.flow.available_methods.contains(&user_req.selected_method) {
            return Err(AppError::BadRequest("Unavailable method".to_string()));
        }


        //* Determine the next stage based on the method
        let next_stage = match user_req.selected_method {
            AuthenticationMethod::Password => SignInState::WithPasswordStage,
            AuthenticationMethod::RecoveryEmailCode => SignInState::ChoseRecoveryEmailStage,
            AuthenticationMethod::RecoveryCode => SignInState::WithRecoveryCodeStage,
            AuthenticationMethod::EmailVerification => SignInState::VerifyEmailStage, // only here we send code
        };

        //* Perform the side effects if the method actually changed
        if session_data.flow.current_method != Some(user_req.selected_method) {
            session_data.flow.current_method = Some(user_req.selected_method);
            session_data.data = Default::default();

            //? If email verification is selected, generate and send code
            if user_req.selected_method == AuthenticationMethod::EmailVerification {
                let user = self.repos.users_repo.get_full_by_id(
                    &session_data.identifier.user_id
                ).await?
                    .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

                let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

                self.email_sender.send_two_factor_code(&user.email, &otp).await;

                session_data.data.email = Some(user.email);
                session_data.data.code = Some(otp);
                session_data.data.attempts = 0;
                session_data.data.last_resent_code_at = Some(chrono::Utc::now());
            }

        }

        //* Update the session stage and assign to next_stage
        session_data.flow.stage = next_stage;

        // Save changes
        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = SelectAuthMethodRes { next_stage };

        Ok(response)
    }

    async fn set_password(&self, app_state: &AppState, user_req: SetPasswordReq) -> Result<SetPasswordRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::WithPasswordStage;
        const CURRENT_METHOD: AuthenticationMethod = AuthenticationMethod::Password;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if !session_data.flow.available_methods.contains(&CURRENT_METHOD) {
            return Err(AppError::BadRequest("Invalid sign-in session method".to_string()));
        }

        if session_data.flow.current_method != Some(CURRENT_METHOD) {
            return Err(AppError::BadRequest("Selected method is correct".to_string()));
        }

        //* Fetch user
        let user = self.repos.users_repo.get_full_by_id(
            &session_data.identifier.user_id
        ).await?
            .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

        //* Verify password
        if !bcrypt::verify(&user_req.password, &user.password)? {
            if session_data.data.attempts >= MAX_ATTEMPTS - 1 {
                self.block_auth_method(&user_req.session_token, &mut session_data).await?;
            }

            session_data.data.attempts += 1;

            self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

            return Err(AppError::BadRequest("Invalid password".to_string()));
        }

        //* NEXT STEPS:
        // Proceed to next stage
        self.proceed_next_stage(&mut session_data, Some(&user)).await?;

        // Save changes
        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = SetPasswordRes { next_stage: session_data.flow.stage };
        Ok(response)
    }

    async fn get_recovery_emails(&self, app_state: &AppState, user_req: GetRecoveryEmailsReq) -> Result<GetRecoveryEmailsRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::ChoseRecoveryEmailStage;
        const CURRENT_METHOD: AuthenticationMethod = AuthenticationMethod::RecoveryEmailCode;

        let Some(session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if !session_data.flow.available_methods.contains(&CURRENT_METHOD) {
            return Err(AppError::BadRequest("Invalid sign-in session method".to_string()));
        }

        if session_data.flow.current_method != Some(CURRENT_METHOD) {
            return Err(AppError::BadRequest("Selected method is correct".to_string()));
        }

        //* Fetch & map recovery emails
        let recovery_emails = self.repos.recovery_emails_repo.get_by_user_id_verified(&session_data.identifier.user_id).await?;

        let emails: Vec<&String> = recovery_emails.iter().map(|recovery_email| &recovery_email.recovery_email ).collect();
        let parsed_emails = mask_emails(&emails);


        let response = GetRecoveryEmailsRes {
            recovery_emails: parsed_emails,
        };
        Ok(response)
    }

    async fn select_recovery_email(&self, app_state: &AppState, user_req: SelectRecoveryEmailReq) -> Result<SelectRecoveryEmailRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::ChoseRecoveryEmailStage;
        const NEXT_STAGE: SignInState = SignInState::VerifyRecoveryEmailCodeStage;
        const CURRENT_METHOD: AuthenticationMethod = AuthenticationMethod::RecoveryEmailCode;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if !session_data.flow.available_methods.contains(&CURRENT_METHOD) {
            return Err(AppError::BadRequest("Invalid sign-in session method".to_string()));
        }

        if session_data.flow.current_method != Some(CURRENT_METHOD) {
            return Err(AppError::BadRequest("Selected method is correct".to_string()));
        }

        //* Get and validate selected email
        let recovery_emails = self.repos.recovery_emails_repo.get_by_user_id_verified(&session_data.identifier.user_id).await?;

        let selected_email = recovery_emails.get(user_req.recovery_email_index)
            .ok_or_else(|| AppError::BadRequest("Invalid recovery email index".to_string()))?;

        //* Generate code, save to session and send email
        let code = format!("{:06}", rand::random::<u32>() % 1_000_000);

        session_data.data.email = Some(selected_email.recovery_email.clone());
        session_data.data.attempts = 0;
        session_data.data.code = Some(code.clone());
        session_data.data.last_resent_code_at = Some(chrono::Utc::now());

        session_data.flow.stage = NEXT_STAGE;

        self.email_sender.send_two_factor_code(&selected_email.recovery_email, &code).await;

        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;


        let response = SelectRecoveryEmailRes { next_stage: NEXT_STAGE };
        Ok(response)
    }

    async fn verify_recovery_email_code(&self, app_state: &AppState, user_req: VerifyRecoveryEmailCodeReq) -> Result<VerifyRecoveryEmailCodeRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::VerifyRecoveryEmailCodeStage;
        const CURRENT_METHOD: AuthenticationMethod = AuthenticationMethod::RecoveryEmailCode;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if !session_data.flow.available_methods.contains(&CURRENT_METHOD) {
            return Err(AppError::BadRequest("Invalid sign-in session method".to_string()));
        }

        if session_data.flow.current_method != Some(CURRENT_METHOD) {
            return Err(AppError::BadRequest("Selected method is correct".to_string()));
        }

        //* Check code
        let code = session_data.data.code.as_ref().ok_or_else(||
            AppError::InternalServerError("Verification code is not set".to_string()) )?;

        if *code != user_req.verification_code {
            if session_data.data.attempts >= MAX_ATTEMPTS - 1 {
                self.block_auth_method(&user_req.session_token, &mut session_data).await?;
            }

            session_data.data.attempts += 1;

            let _ = self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

            return Err(AppError::BadRequest("Invalid verification code".to_string()));
        }

        //* NEXT STEPS:
        self.proceed_next_stage(&mut session_data, None).await?;

        // Save changes
        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = VerifyRecoveryEmailCodeRes {
            next_stage: session_data.flow.stage,
        };
        Ok(response)
    }

    async fn set_recovery_code(&self, app_state: &AppState, user_req: SetRecoveryCodeReq) -> Result<SetRecoveryCodeRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::WithRecoveryCodeStage;
        const CURRENT_METHOD: AuthenticationMethod = AuthenticationMethod::RecoveryCode;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if !session_data.flow.available_methods.contains(&CURRENT_METHOD) {
            return Err(AppError::BadRequest("Invalid sign-in session method".to_string()));
        }

        if session_data.flow.current_method != Some(CURRENT_METHOD) {
            return Err(AppError::BadRequest("Selected method is correct".to_string()));
        }

        //* Fetch code and validate
        let res = self.repos.recovery_codes_repo.get_by_user_id_and_code(&session_data.identifier.user_id, &user_req.recovery_code).await?;
        match res {
            Some(recovery_code) => {
                // Mark code as used
                self.repos.recovery_codes_repo.delete_by_id(&recovery_code.id.to_string()).await?;
            },
            None => {
                if session_data.data.attempts >= MAX_ATTEMPTS - 1 {
                    self.block_auth_method(&user_req.session_token, &mut session_data).await?;
                }

                session_data.data.attempts += 1;

                self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

                return Err(AppError::BadRequest("Invalid recovery code".to_string()));
            }
        }

        //* NEXT STEPS:
        // Proceed to next stage
        self.proceed_next_stage(&mut session_data, None).await?;

        // Save changes
        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = SetRecoveryCodeRes { next_stage: session_data.flow.stage };
        Ok(response)
    }

    async fn verify_email(&self, app_state: &AppState, user_req: VerifyEmailReq) -> Result<VerifyEmailRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::VerifyEmailStage;
        const CURRENT_METHOD: AuthenticationMethod = AuthenticationMethod::EmailVerification;
        const MAX_ATTEMPTS: u8 = 5;

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if !session_data.flow.available_methods.contains(&CURRENT_METHOD) {
            return Err(AppError::BadRequest("Invalid sign-in session method".to_string()));
        }

        if session_data.flow.current_method != Some(CURRENT_METHOD) {
            return Err(AppError::BadRequest("Selected method is correct".to_string()));
        }

        //* Check code
        let code = session_data.data.code.as_ref().ok_or_else(||
            AppError::InternalServerError("Verification code is not set".to_string()) )?;

        if *code != user_req.verification_code {
            if session_data.data.attempts >= MAX_ATTEMPTS - 1 {
                self.block_auth_method(&user_req.session_token, &mut session_data).await?;
            }

            session_data.data.attempts += 1;

            let _ = self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

            return Err(AppError::BadRequest("Invalid verification code".to_string()));
        }


        //* NEXT STEPS:
        // Proceed to next stage
        self.proceed_next_stage(&mut session_data, None).await?;

        // Save changes
        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;

        let response = VerifyEmailRes { next_stage: session_data.flow.stage };

        Ok(response)
    }

    async fn resend_code(&self, app_state: &AppState, user_req: ResendCodeReq) -> Result<ResendCodeRes, AppError> {
        const AVAILABLE_STAGES: &[SignInState] = &[ SignInState::VerifyEmailStage, SignInState::VerifyRecoveryEmailCodeStage ];
        const AVAILABLE_METHODS: &[AuthenticationMethod] = &[ AuthenticationMethod::EmailVerification, AuthenticationMethod::RecoveryEmailCode ];
        const TIME_DIFFERENCE: u8 = 60; // seconds

        let Some(mut session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        //* Validate session stage and method
        if !AVAILABLE_STAGES.contains(&session_data.flow.stage)  {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        if !session_data.flow.available_methods.iter()
            .any(|item| AVAILABLE_METHODS.contains(item))
        {
            return Err(AppError::BadRequest("Invalid sign-in session method".to_string()));
        }

        let current_method = session_data.flow.current_method
            .ok_or_else(|| AppError::BadRequest("Current authentication method is not set".to_string()))?;

        if !AVAILABLE_METHODS.contains(&current_method) {
            return Err(AppError::BadRequest("Selected method is correct".to_string()));
        }

        //* Check time difference between now and last resent code time
        let last_resent = session_data.data.last_resent_code_at
            .ok_or_else(|| AppError::BadRequest("Last resent code time is not set".to_string()))?;

        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last_resent);

        if duration.num_seconds() < TIME_DIFFERENCE as i64 {
            return Err(AppError::BadRequest(format!("You can resend the code after {} seconds", TIME_DIFFERENCE)));
        }

        //* Generate new code, save to session and send email
        let email = session_data.data.email.as_ref()
            .ok_or_else(|| AppError::InternalServerError("Email not found".to_string()))?;

        let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

        self.email_sender.send_two_factor_code(email, &otp).await;

        session_data.data.code = Some(otp);
        session_data.data.attempts = 0;
        session_data.data.last_resent_code_at = Some(chrono::Utc::now());

        self.repos.signin_repo.update(&user_req.session_token, &session_data).await?;


        let response = ResendCodeRes;
        Ok(response)
    }
    async fn final_session(&self, app_state: &AppState, user_req: FinalizeSignInReq) -> Result<FinalizeSignInRes, AppError> {
        const CURRENT_STAGE: SignInState = SignInState::Redirect;

        let Some(session_data) = self.repos.signin_repo.get(&user_req.session_token).await? else {
            return Err(AppError::BadRequest("Sign-in session not found".to_string()))
        };

        if session_data.flow.stage != CURRENT_STAGE {
            return Err(AppError::BadRequest("Invalid sign-in session stage".to_string()));
        }

        self.check_global_max_attempts(&user_req.session_token, &session_data).await?;

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

    /// - Clean data buffer
    /// - Mark method as satisfied and remove from available methods
    /// - If MFA enabled and this is first satisfied method, select first available method and go there
    /// - Else, proceed to finalization
    async fn proceed_next_stage(&self, session_data: &mut SignInSession, user: Option<&User>) -> Result<(), AppError> {
        let current_method = session_data.flow.current_method
            .ok_or_else(|| AppError::InternalServerError("Current authentication method is not set".to_string()))?;

        // Mark password method as satisfied
        session_data.flow.satisfied_methods.push(current_method);
        let i = session_data.flow.available_methods.iter()
            .position(|m| *m == current_method)
            .ok_or_else(|| AppError::InternalServerError("Authentication method not found".to_string()))?;
        session_data.flow.available_methods.remove(i);

        // Swap all blocked methods to available
        session_data.flow.available_methods.append(&mut session_data.flow.blocked_methods);
        session_data.flow.blocked_methods.clear();
        session_data.flow.incorrect_attempts = 0;

        // Clear previous method data buffer
        session_data.data = Default::default();

        // Conditional logic for MFA
        let mfa_enabled = session_data.identifier.mfa_enabled;
        let first_time = session_data.flow.satisfied_methods.len() == 1;

        // We are here first time. We have to redirect to the other authentication method after this route
        // Or if MFA is disabled, we can proceed to finalization
        // Or if it's second time, we can proceed to finalization
        if mfa_enabled && first_time {
            // select method to proceed
            session_data.flow.available_methods.sort();
            let first_method = session_data.flow.available_methods.first()
                .ok_or_else(|| AppError::BadRequest("No methods to confirm your identity".to_string()))?;

            match first_method {
                AuthenticationMethod::EmailVerification => {
                    let user = match user {
                        Some(u) => u,
                        None => {
                            &self.repos.users_repo.get_full_by_id(
                                &session_data.identifier.user_id
                            ).await?
                                .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?
                        }
                    };

                    let otp = format!("{:06}", rand::random::<u32>() % 1_000_000);

                    self.email_sender.send_two_factor_code(&user.email, &otp).await;

                    session_data.data.email = Some(user.email.clone());
                    session_data.data.code = Some(otp);
                    session_data.data.attempts = 0;
                    session_data.data.last_resent_code_at = Some(chrono::Utc::now());

                    session_data.flow.current_method = Some(AuthenticationMethod::EmailVerification);
                    session_data.flow.stage = SignInState::VerifyEmailStage;
                },
                AuthenticationMethod::RecoveryEmailCode => {
                    session_data.flow.current_method = Some(AuthenticationMethod::RecoveryEmailCode);
                    session_data.flow.stage = SignInState::ChoseRecoveryEmailStage;
                },
                AuthenticationMethod::RecoveryCode => {
                    session_data.flow.current_method = Some(AuthenticationMethod::RecoveryCode);
                    session_data.flow.stage = SignInState::WithRecoveryCodeStage;
                },
                AuthenticationMethod::Password => {
                    session_data.flow.current_method = Some(AuthenticationMethod::Password);
                    session_data.flow.stage = SignInState::WithPasswordStage;
                }
            }

        }
        else {
            // proceed to finalization
            session_data.flow.current_method = None;
            session_data.flow.stage = SignInState::Redirect;
        }

        Ok(())
    }


    async fn block_auth_method(&self, session_token: &str, session_data: &mut SignInSession) -> Result<(), AppError> {
        let current_method = session_data.flow.current_method
            .ok_or_else(|| AppError::InternalServerError("Current authentication method is not set".to_string()))?;

        session_data.flow.incorrect_attempts += 1;

        let i = session_data.flow.available_methods.iter()
            .position(|m| *m == current_method)
            .ok_or_else(|| AppError::InternalServerError("Authentication method not found".to_string()))?;
        session_data.flow.available_methods.remove(i);
        session_data.flow.blocked_methods.push(current_method);
        session_data.flow.current_method = None;
        session_data.flow.stage = SignInState::SelectAuthenticationMethodsStage;

        session_data.data = Default::default();

        self.repos.signin_repo.update(session_token, session_data).await?;

        Err(AppError::BadRequest("This authentication method is blocked due to too many incorrect attempts. Please select another method".to_string()))
    }

    async fn check_global_max_attempts(&self, session_token: &str, session_data: &SignInSession) -> Result<(), AppError> {
        const GLOBAL_MAX_ATTEMPTS: u8 = 3;

        if session_data.flow.incorrect_attempts >= GLOBAL_MAX_ATTEMPTS {
            //TODO: check if user blocked by ip and user_id
            //TODO: if so, block by ip on long time
            //TODO: else, block ip by user_id on short time

            // delete session
            self.repos.signin_repo.delete(session_token).await?;

            return Err(AppError::BadRequest("Too many incorrect attempts. Please try again later.".to_string()));
        }

        Ok(())
    }
}