use log::info;
use crate::services::email_sender::EmailSender;

pub struct DebugEmailSender;

#[async_trait::async_trait]
impl EmailSender for DebugEmailSender {
    async fn test(&self) {
        info!("Test debug email sender");
    }

    async fn send_confirmation_sign_up(&self, to: &str, code: &str) {
        info!("Sending confirmation signup email_sender to {}: {}", to, code);
    }

    async fn send_two_factor_code(&self, to: &str, code: &str) {
        info!("Sending two-factor code email_sender to {}: {}", to, code);
    }

    async fn send_forgot_password(&self, to: &str, code: &str) {
        info!("Sending confirmation signup email_sender to {}: {}", to, code);
    }

    async fn send_change_email(&self, to: &str, code: &str) {
        info!("Sending two-factor code email_sender to {}: {}", to, code);
    }
}

impl DebugEmailSender {
    pub fn new() -> Self { Self }
}