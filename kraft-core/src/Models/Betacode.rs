use serde::{self, Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct Betacode {
    pub betacode: String,
    pub enabled: bool,
}
