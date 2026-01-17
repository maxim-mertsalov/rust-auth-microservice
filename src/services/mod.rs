use std::sync::Arc;
use crate::repositories::AppRepositories;
use crate::services::auth::AuthServices;
use crate::services::email_sender::EmailSender;
use crate::services::test::TestService;

pub mod auth;
pub mod test;
pub mod email_sender;



pub struct AppServices {
    pub auth_services: AuthServices,
    pub test_service: TestService,
}

impl AppServices {
    pub fn new(repos: AppRepositories, email_sender: Arc<dyn EmailSender + Send + Sync>) -> Self {
        AppServices {
            auth_services: AuthServices::new(Arc::new(repos.auth_repos), Arc::clone(&email_sender)),
            test_service: TestService::new(Arc::new(repos.test_repo), email_sender),
        }
    }
}
