use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use log::error;
use crate::errors::message_sender_error::MessageSenderError;
use crate::services::email_sender::EmailSender;

pub struct SMTPEmailSender {
    smtp_transport: AsyncSmtpTransport<Tokio1Executor>,
    email_sender: Mailbox,
}

#[async_trait::async_trait]
impl EmailSender for SMTPEmailSender {
    async fn test(&self) {
        let _ = self.send_email("test@example.com", "Test smtp server", "TEST").await;
    }

    async fn send_confirmation_sign_up(&self, to: &str, code: &str) {
        let _ = self.send_email(&*to, "Sign up confirmation", code).await;
    }

    async fn send_two_factor_code(&self, to: &str, code: &str) {
        let _ = self.send_email(&*to, "Sign in confirmation", code).await;
    }

    async fn send_forgot_password(&self, to: &str, code: &str) {
        let _ = self.send_email(&*to, "Forgot password reset code", code).await;
    }

    async fn send_change_email(&self, to: &str, code: &str) {
        let _ = self.send_email(&*to, "Change email", code).await;
    }
}

impl SMTPEmailSender {
    pub fn new(smtp_url: &str, email: Mailbox) -> Result<Self, MessageSenderError> {
        let smtp_transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(&*smtp_url)?.build();

        Ok(SMTPEmailSender { smtp_transport, email_sender: email })
    }

    pub async fn send_email(&self, to: &str, subj: &str, message: &str) -> Result<(), MessageSenderError> {
        let email_addr = match to.parse() {
            Ok(addr) => addr,
            Err(err) => {
                error!("Could not parse email address: {}", err);
                return Err(MessageSenderError::Other(format!("Could not parse email address: {}", err))) }
        };


        let email = Message::builder()
            .from(self.email_sender.clone())
            .to(Mailbox::new(None, email_addr))
            .subject(subj)
            .header(ContentType::TEXT_PLAIN)
            .body(message.to_owned())?;

        self.smtp_transport.send(email).await?;

        Ok(())
    }
}