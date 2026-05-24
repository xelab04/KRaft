use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct PasswordChange {
    pub current_password: String,
    pub new_password: String
}

// Generate hashed password for test
#[derive(Deserialize)]
pub struct PasswordParams{
    pub user_password: String
}
