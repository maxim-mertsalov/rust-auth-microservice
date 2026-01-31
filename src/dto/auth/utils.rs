


#[derive(Debug, Clone)]
pub struct TokenCreatorParams {
    pub user_id: sqlx::types::Uuid,
    pub days_to_inactive: i32,
}