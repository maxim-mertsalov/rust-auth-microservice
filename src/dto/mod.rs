use serde::Serialize;

pub mod auth;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: Option<T>
}

#[derive(Debug, Serialize, Clone)]
pub struct ApiErrorResponse {
    pub status: String,
    pub message: String,
    pub errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Serialize, Clone)]
#[derive(PartialEq)]
pub struct ApiError {
    pub field: String,
    pub message: String,
}
