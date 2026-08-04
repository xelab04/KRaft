use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub struct namespace_resources {
    pub cpu: i32,
    pub memory: i32,
    pub storage: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
pub struct cluster_resources {
    pub cpu: i32,
    pub memory: i32,
    pub storage: i32,
}
