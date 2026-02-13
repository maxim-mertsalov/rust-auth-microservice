use std::env;
use lettre::message::Mailbox;
use crate::services::email_sender::debug_sender::DebugEmailSender;
use crate::services::email_sender::EmailSender;
use crate::services::email_sender::smtp_sender::SMTPEmailSender;

pub struct AppConfig{
    // Server
    pub port: u16,

    // Secret keys
    pub secret_key: String, // used for signing JWT tokens
    pub sudo_secret_key: String, // used for signing sudo tokens
    pub hash_secret: String, // used for salting password hashes

    // Database
    pub pg_url: String,
    pub redis_url: String,

    // Boolean flags for enabling/disabling features
    pub require_email_confirmation: bool, // if true email_sender confirmation is required after registration
    pub enable_email_reset: bool, // if true changing emails is enabled

    // Email sender
    pub email_sender: Box<dyn EmailSender + Send + Sync>
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok(); // Load .env file

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .unwrap_or(8000);

        // Database URLs
        let pg_url = env::var("PG_URL").expect("PG_URL must be set");
        let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

        // Secret key
        let secret_key = env::var("SECRET").expect("SECRET must be set");

        let sudo_secret_key = env::var("SUDO_SECRET").unwrap_or_else(|_| secret_key.clone());

        // Hash secret
        let hash_secret = env::var("HASH_SECRET").unwrap_or_else(|_| secret_key.clone());

        // Feature flags
        let require_email_confirmation = env::var("REQUIRE_EMAIL_CONFIRMATION")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let enable_email_reset = env::var("ENABLE_EMAIL_RESET")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let email_sender: Box<dyn EmailSender + Send + Sync> = Self::get_email_sender();

        Self {
            port,
            pg_url,
            redis_url,
            secret_key,
            sudo_secret_key,
            hash_secret,
            require_email_confirmation,
            enable_email_reset,
            email_sender
        }
    }

    fn get_email_sender() -> Box<dyn EmailSender + Send + Sync> {
        let email_sender_type = env::var("EMAIL_SENDER_TYPE").expect("EMAIL_SENDER_TYPE must be set");

        match email_sender_type.as_str() {
            "debug" => Box::new(DebugEmailSender::new()),
            "smtp" => {
                let smtp_url = env::var("SMTP_URL").expect("SMTP_URL must be set");
                let from_name_env = env::var("SMTP_FROM_NAME").unwrap_or("".to_string());
                let from_email = env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL must be set");

                let from_name = if from_name_env.is_empty() { None } else { Some(from_name_env) };

                let mailbox = Mailbox::new(from_name, from_email.parse().unwrap());

                Box::new(SMTPEmailSender::new(&smtp_url, mailbox).unwrap())
            }
            // "rabbitmq" =>
            // "server" =>
            _ => panic!("EMAIL_SENDER_TYPE may be only 'debug', 'smtp', 'rabbitmq' or 'server'"),
        }
    }
}