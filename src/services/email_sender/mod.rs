
pub mod smtp_sender;
pub mod debug_sender;

#[async_trait::async_trait]
pub trait EmailSender {
    async fn test(&self);
    async fn send_confirmation_sign_up(&self, to: &str, code: &str);
    async fn send_two_factor_code(&self, to: &str, code: &str);
    async fn send_forgot_password(&self, to: &str, code: &str);
    async fn send_change_email(&self, to: &str, code: &str);
}
