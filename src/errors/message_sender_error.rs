#[derive(Debug)]
pub enum MessageSenderError {
    IncorrectCred(String),
    SmtpError(String),
    Other(String),
}

impl From<lettre::error::Error> for MessageSenderError {
    fn from(err: lettre::error::Error) -> Self {
        MessageSenderError::Other(err.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for MessageSenderError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        MessageSenderError::SmtpError(err.to_string())
    }
}

