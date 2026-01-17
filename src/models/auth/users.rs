use serde::{Serialize};
use sqlx::FromRow;

/// users table
#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: sqlx::types::Uuid,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub is_verified: bool,
    pub is_two_factor: bool,
    pub days_to_inactive: i32, // after how many days of inactivity the user will be marked as inactive
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ----- Get user -----
#[derive(Debug, Serialize, FromRow)]
pub struct MidFieldsUser {
    pub id: sqlx::types::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_verified: bool,
    pub is_two_factor: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}


// ----- Get users -----
#[derive(Debug, Serialize, FromRow)]
pub struct MinFieldsUser {
    pub id: sqlx::types::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}
