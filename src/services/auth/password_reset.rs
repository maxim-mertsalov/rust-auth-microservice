// use std::sync::Arc;
// use crate::dto::auth::password_resets::{ChangePasswordRequest, ForgotPasswordRequest, ForgotPasswordTypeResponse};
// use crate::errors::app_error::AppError;
// use crate::repositories::auth::AuthRepositories;
// use crate::services::email_sender::EmailSender;
// use crate::state::app_state::AppState;
//
// #[async_trait::async_trait]
// pub trait IPasswordResetService {
//     /// ----- Change password by /password/change when logged in -----
//     async fn change_password(&self, req: ChangePasswordRequest) -> Result<(), AppError>;
//
//     /// ----- Forgot password by /password/forgot -----
//     async fn forgot_password(&self, req: ForgotPasswordRequest) -> Result<ForgotPasswordTypeResponse, AppError>;
//
//     /// ----- Confirm forgotten password by /password/reset -----
//     async fn reset_password(&self, code: String) -> Result<(), AppError>;
// }
//
// pub struct PasswordResetService {
//     repos: Arc<AuthRepositories>,
//     email_sender: Arc<dyn EmailSender + Send + Sync>,
// }