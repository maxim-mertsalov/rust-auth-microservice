use std::sync::Arc;
use crate::repositories::auth::{AuthRepositories};
use crate::services::auth::session::SessionService;
use crate::services::auth::signin::SignInService;
use crate::services::auth::signup::SignUpService;
use crate::services::auth::user_management::UserManagementService;
use crate::services::auth::utils::{AuthUtilsService};
use crate::services::email_sender::EmailSender;

pub mod user_management;
pub mod session;
mod utils;
pub mod signup;
pub mod signin;

pub struct AuthServices {
    pub signup_service: Arc<SignUpService> ,
    pub signin_service: Arc<SignInService> ,
    pub user_management_service: Arc<UserManagementService>,
    pub session_service: Arc<SessionService>,
}

impl AuthServices {
    pub fn new(repos: Arc<AuthRepositories>, email_sender: Arc<dyn EmailSender + Send + Sync>) -> Self {

        let utils = Arc::new(AuthUtilsService::new(repos.clone()));

        Self {
            signup_service: Arc::new(SignUpService::new(repos.clone(), utils.clone(), Arc::clone(&email_sender) )),
            signin_service: Arc::new(SignInService::new(repos.clone(), utils, Arc::clone(&email_sender))),
            user_management_service: Arc::new(UserManagementService::new(repos.clone())),
            session_service: Arc::new(SessionService::new(repos)),
        }
    }
}