use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Domain {
    pub user_id: i32,
    pub domain: String,
    pub token_id: String,
}
