#[derive(Debug)]
pub enum MessageSenderError {
    IncorrectCred(String),
    SmtpError(String),
    Other(String),
}

impl From<lettre::error::Error> for MessageSenderError {
    fn from(err: lettre::error::Error) -> Self {
        // match err {
        //     Error::MissingFrom => {}
        //     Error::MissingTo => {}
        //     Error::TooManyFrom => {}
        //     Error::EmailMissingAt => {}
        //     Error::EmailMissingLocalPart => {}
        //     Error::EmailMissingDomain => {}
        //     Error::CannotParseFilename => {}
        //     Error::Io(_) => {}
        //     Error::NonAsciiChars => {}
        // }
        MessageSenderError::Other(err.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for MessageSenderError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        MessageSenderError::SmtpError(err.to_string())
    }
}

