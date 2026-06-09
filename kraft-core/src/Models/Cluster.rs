use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use serde::{self, Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Cluster {
    pub id: Option<i32>,
    pub name: String,
    pub endpoint: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct ClusterCreateForm {
    pub id: Option<i64>,
    pub name: String,
    pub tlssan_array: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClusterResourceConfig {
    pub cluster_resources: ClusterResources,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClusterResources {
    pub servers: ResourceCategory,
    pub workers: ResourceCategory,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResourceCategory {
    pub requests: ResourceValues,
    pub limits: ResourceValues,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResourceValues {
    pub cpu: IntOrString,
    pub memory: IntOrString,
}
