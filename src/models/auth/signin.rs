use serde::{Deserialize, Serialize};
use crate::models::auth::utils::{AuthFlowMetadata};

/// signup:{token}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SignInSession {
    pub metadata: AuthFlowMetadata,
    pub identifier: SignInFlowIdentifier,
    pub flow: SignInFlow,
    pub data: SignInFlowData,
}

pub const SIGNIN_SESSION_PREFIX: &str = "signin";
pub const SIGNIN_SESSION_LIFETIME: i64 = 20; // reset every time when request anything

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub enum AuthenticationMethod {
    #[serde(rename = "password")]
    Password = 1,
    #[serde(rename = "email_verification")]
    EmailVerification = 2,
    #[serde(rename = "recovery_email_verification")]
    RecoveryEmailCode = 3,
    #[serde(rename = "recovery_code")]
    RecoveryCode = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Copy)]
pub enum SignInState {
    #[default]
    InitWithEmailStage,

    SelectAuthenticationMethodsStage,

    // Authentication methods stages
    // 1. Password (default)
    WithPasswordStage,

    // 2. Recovery Email Code
    ChoseRecoveryEmailStage, // chose if many recovery emails or skip if only one
    VerifyRecoveryEmailCodeStage,

    // 3. Recovery Code
    WithRecoveryCodeStage,

    // 4. Email Verification
    VerifyEmailStage, // default second step for MFA


    Redirect
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct SignInFlowIdentifier {
    pub user_id: String,
    pub mfa_enabled: bool,
    pub requested_sudo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignInFlow {
    // Flow info
    pub satisfied_methods: Vec<AuthenticationMethod>,
    pub available_methods: Vec<AuthenticationMethod>,
    pub blocked_methods: Vec<AuthenticationMethod>,
    pub incorrect_attempts: u8,
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