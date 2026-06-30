use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// let's not store the token itself in the db, right?
#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct Token {
    pub token_id: String,
    pub user_id: i32,
}
