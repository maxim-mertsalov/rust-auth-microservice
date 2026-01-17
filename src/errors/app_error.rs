use std::fmt;
use std::fmt::{Debug, Display};
use actix_web::{HttpResponse, HttpResponseBuilder, Responder, ResponseError};
use actix_web::http::StatusCode;
use bcrypt::BcryptError;
use validator::{ValidationErrors, ValidationErrorsKind};
use crate::dto::{ApiError, ApiErrorResponse};
use crate::errors::db_error::DbError;
use crate::errors::hash_error::HashError;

#[derive(Debug)]
pub enum AppError {
    NoContent(String),
    BadRequest(String), // Validation error, e.g. invalid input
    Unauthorized(String), // All with authentication
    NotFound(String),
    InternalServerError(String),
    ExpiredAccessToken,
    Other(String),
    ValidationError(Vec<ApiError>),
    Custom((ApiErrorResponse, StatusCode))
}

impl PartialEq<Self> for AppError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AppError::NoContent(a), AppError::NoContent(b)) => a == b,
            (AppError::BadRequest(a), AppError::BadRequest(b)) => a == b,
            (AppError::Unauthorized(a), AppError::Unauthorized(b)) => a == b,
            (AppError::NotFound(a), AppError::NotFound(b)) => a == b,
            (AppError::InternalServerError(a), AppError::InternalServerError(b)) => a == b,
            (AppError::Other(a), AppError::Other(b)) => a == b,
            (AppError::ValidationError(a), AppError::ValidationError(b)) => a == b,
            (AppError::ExpiredAccessToken, AppError::ExpiredAccessToken) => true,
            _ => false,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NoContent(msg) => write!(f, "No Content: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            AppError::Other(msg) => write!(f, "Other Error: {}", msg),
            AppError::ValidationError(err) => write!(f, "Validation Errors: {:?}", err),
            AppError::ExpiredAccessToken => write!(f, "Access Token expired"),
            AppError::Custom(msg) => write!(f, "{:?}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let mut api_error_response = ApiErrorResponse {
            status: "error".to_string(),
            message: self.to_string(),
            errors: Some(Vec::new()),
        };

        match self {
            AppError::NoContent(err) => {
                api_error_response.message = err.to_string();
                HttpResponse::NotFound().json(api_error_response)
            },
            AppError::BadRequest(err) => {
                api_error_response.message = err.to_string();
                HttpResponse::BadRequest().json(api_error_response)
            },
            AppError::Unauthorized(err) => {
                api_error_response.message = err.to_string();
                HttpResponse::Unauthorized().json(api_error_response)
            },
            AppError::NotFound(err) => {
                api_error_response.message = err.to_string();
                HttpResponse::NotFound().json(api_error_response)
            },
            AppError::InternalServerError(err) => {
                api_error_response.message = err.to_string();
                HttpResponse::InternalServerError().json(api_error_response)
            },
            AppError::Other(err) => {
                api_error_response.message = err.to_string();
                HttpResponse::InternalServerError().json(api_error_response)
            },
            AppError::ValidationError(errs) => {
                api_error_response.errors = Some(errs.clone());
                api_error_response.message = "Validation error".to_string();

                HttpResponse::BadRequest().json(api_error_response)
            },
            AppError::ExpiredAccessToken => {
                api_error_response.message = "Access token has expired".to_string();
                HttpResponse::Unauthorized().json(api_error_response)
            },
            AppError::Custom(err) => {
                api_error_response = err.0.clone();
                HttpResponseBuilder::new(err.1).json(api_error_response)
            }
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.clone().into_kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken => AppError::Unauthorized("Invalid token".to_string()),
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => AppError::Unauthorized("Invalid issuer".to_string()),
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::ExpiredAccessToken,
            _ => AppError::InternalServerError(err.to_string()),
        }
    }
}

impl From<BcryptError> for AppError {
    fn from(err: BcryptError) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}


impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl From<DbError> for AppError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound(msg) => AppError::NotFound(msg),
            DbError::ConnectionError(msg) => AppError::InternalServerError(msg),
            DbError::Timeout(msg) => AppError::InternalServerError(msg),
            DbError::FromDatabase(msg) => AppError::BadRequest(msg),
            DbError::Unknown(msg) => AppError::InternalServerError(msg),
            DbError::QueryError(msg) => AppError::InternalServerError(msg)
        }
    }
}

impl From<ValidationErrors> for AppError {
    fn from(errs: ValidationErrors) -> Self {
        let mut errors = Vec::new();

        errs.0.iter().for_each(|(field, errors_kind)| {
            if let ValidationErrorsKind::Field( messages ) = errors_kind {
                messages.iter().for_each(|message| {
                    if let Some(msg) = &message.message {
                        let error = ApiError {
                            field: field.to_string(),
                            message: msg.to_string(),
                        };
                        errors.push(error);
                        return;
                    }
                });
            }
        });

        AppError::ValidationError(errors)
    }
}

impl From<HashError> for AppError {
    fn from(err: HashError) -> Self {
        match err {
            HashError::InvalidHash(msg) => AppError::BadRequest(msg),
            HashError::HashingFailed(msg) => AppError::InternalServerError(msg),
            HashError::Other(msg) => AppError::InternalServerError(msg),
        }
    }
}