use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct namespace_resources {
    pub cpu: i32,
    pub memory: i32,
    pub storage: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct cluster_resources {
    pub cpu: i32,
    pub memory: i32,
    pub storage: i32,
}
