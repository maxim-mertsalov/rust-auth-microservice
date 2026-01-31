use serde::{Deserialize, Serialize};
use crate::models::auth::utils::{FlowMetadata};

/// signup:{token}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SignInSession {
    pub metadata: FlowMetadata,
    pub identifier: SignInFlowIdentifier,
    pub flow: SignInFlow,
    pub data: SignInFlowData,
}

pub const SIGNIN_SESSION_PREFIX: &str = "signin";
pub const SIGNIN_SESSION_LIFETIME: i64 = 20; // reset every time when request anything

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthenticationMethod {
    Password,
    RecoveryEmailCode,
    RecoveryCode,
    EmailVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum SignInState {
    #[default]
    InitWithEmailStage,

    SetAuthenticationMethodsStage,

    // Authentication methods stages
    WithPasswordStage, // default first step

    ChoseRecoveryEmailStage,
    VerifyRecoveryEmailCodeStage,

    WithRecoveryCodeStage,
    VerifyEmailStage, // default second step for MFA


    Redirect
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct SignInFlowIdentifier {
    pub user_id: String,
    pub mfa_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignInFlow {
    // Flow info
    pub satisfied_methods: Vec<AuthenticationMethod>,
    pub available_methods: Vec<AuthenticationMethod>,
    pub current_method: Option<AuthenticationMethod>,

    // Current stage
    pub stage: SignInState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignInFlowData {
    // Verification info
    pub code: Option<String>, // for email verification or recovery email code
    pub email: Option<String>, // for recovery email code
    pub attempts: u8,
    pub last_resent_code_at: Option<chrono::DateTime<chrono::Utc>>,
}