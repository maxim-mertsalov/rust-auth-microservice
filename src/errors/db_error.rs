use bb8::RunError;
use bb8_redis::RedisConnectionManager;
use redis::{ErrorKind, RedisError};
use crate::errors::app_error::AppError;

#[derive(Debug)]
pub enum DbError {
    ConnectionError(String),
    QueryError(String),
    NotFound(String),
    Timeout(String),
    Unknown(String),
    FromDatabase(String),
}


impl From<RunError<RedisConnectionManager>> for DbError {
    fn from(err: RunError<RedisConnectionManager>) -> Self {
        match err {
            RunError::User(_) => DbError::ConnectionError("User error in Redis connection manager".to_string()),
            RunError::TimedOut => DbError::Timeout("Redis connection timed out".to_string()),
        }
    }
}

impl From<RunError<RedisError>> for DbError {
    fn from(err: RunError<RedisError>) -> Self {
        match err {
            RunError::User(err) => DbError::ConnectionError(err.to_string()),
            RunError::TimedOut => DbError::Timeout("Redis operation timed out".to_string()),
        }
    }
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound("Row not found".to_string()),
            sqlx::Error::Database(err) => DbError::FromDatabase(err.to_string()),
            sqlx::Error::Io(err) => DbError::FromDatabase(err.to_string()),
            sqlx::Error::Tls(err) => DbError::ConnectionError(err.to_string()),
            sqlx::Error::Protocol(err) => DbError::ConnectionError(err.to_string()),
            sqlx::Error::Configuration(err) => DbError::ConnectionError(err.to_string()),
            sqlx::Error::ColumnNotFound(err) => DbError::NotFound(err.to_string()),
            _ => DbError::Unknown(err.to_string()),
        }
    }
}

impl From<RedisError> for DbError {
    fn from(err: RedisError) -> Self {
        match err.kind() {
            ErrorKind::AuthenticationFailed => { DbError::ConnectionError(format!("Redis Authentication Failed {}", err)) }
            _ => { DbError::Unknown(format!("Redis Error {}", err)) }
        }
    }
}

impl From<uuid::Error> for DbError {
    fn from(err: uuid::Error) -> Self {
        DbError::Unknown(err.to_string())
    }
}

impl From<AppError> for DbError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound(msg) => DbError::NotFound(msg),
            AppError::NoContent(msg) => DbError::NotFound(msg),
            _ => DbError::Unknown(err.to_string()),
        }
    }
}