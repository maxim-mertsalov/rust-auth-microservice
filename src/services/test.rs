use std::sync::Arc;
use crate::errors::db_error::DbError;
use crate::repositories::TestRepository;
use crate::services::email_sender::EmailSender;

#[derive(Clone)]
pub struct TestService {
    repos: Arc<TestRepository>,
    email_sender: Arc<dyn EmailSender + Sync + Send>,
}

impl TestService {
    pub fn new(test_repo: Arc<TestRepository>, email_sender: Arc<dyn EmailSender + Sync + Send>) -> TestService {
        TestService {
            repos: test_repo,
            email_sender,
        }
    }
}

impl TestService {
    pub async fn test_postgres(&self) -> Result<String, DbError> {
        self.repos.test_postgres().await
    }
    pub async fn test_redis(&self) -> Result<String, DbError> {
        self.repos.test_redis().await
    }
    pub async fn test_email_sender(&self) -> Result<String, DbError> {
        self.email_sender.send_confirmation_sign_up(&"hello@gmail.com", &"123").await;
        Ok("Test email sender".to_string())
    }
}