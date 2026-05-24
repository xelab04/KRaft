use sqlx::{FromRow};
use serde::{self, Serialize, Deserialize};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Cluster {
    pub id: Option<i32>,
    pub name: String,
    pub endpoint: Option<String>
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct ClusterCreateForm {
    pub id: Option<i64>,
    pub name: String,
    pub tlssan_array: Option<Vec<String>>
}
