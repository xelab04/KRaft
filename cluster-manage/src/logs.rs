use std::collections::BTreeMap;

use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx;
use sqlx::prelude::FromRow;
use sqlx::MySqlPool;

use k3k_rs;
use kube::Client;
use kube::api::LogParams;

use random_word::Lang;
use tokio::fs;

use crate::jwt;
use crate::validatename;
use crate::AppConfig;
use crate::tlssan;
use crate::ingress;


#[derive(Serialize, Deserialize, FromRow)]
pub struct Cluster {
    name: String
}

#[derive(Serialize, Deserialize)]
pub struct LogsType {
    logtype: String
}

#[get("/api/logs")]
pub async fn getlogs(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    Json(cluster): Json<ClusterCreateForm>,
    query: web::Query<LogsType>
) -> HttpResponse {

    let jwt = jwt::extract_user_id_from_jwt(&req);
    let cluster_name = format!("{}-{}", user_id, cluster.name);
    let namespace = format!("k3k-{}", cluster_name);

    let logtype = &query.logtype;

    // assume that for testing purposes the User's ID is 0
    let mut user_id: String = String::from("0");
    match jwt {
        Ok(id) => {
            user_id = Some(id).unwrap();
        }
        Err(e) => {
            println!("Error: {:?}", e);
            if config.environment == "PROD" {
                return HttpResponse::Unauthorized().json(json!({"status": "error", "message": "Unauthorized"}));
            }
        }
    };

    // check user owns that cluster
    let cluster_id_from_db: i64 = sqlx::query_scalar!("SELECT id FROM clusters WHERE user_id = ? AND cluster_name = ?")
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(&*pool)
        .await

    let mut cluster_id;
    match cluster_id_from_db {
        Ok(id) => {
            cluster_id = id;
        }
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().json(json!({"status": "error", "message": "Cluster not found"}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({"status": "error", "message": "Failed to check cluster"}))
        }
    }

    // default to server
    if logtype == "agent" {
        let logs_returned = k3k_rs::logs::agent(&client, &cluster_name, &namespace).await
    } else {
        let logs_returned = k3k_rs::logs::server(&client, &cluster_name, &namespace).await
    }

    match logs_returned {
        Ok(logs) => {
            HttpResponse::Ok().json(json!({"status": "success", "logs": logs}))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({"status": "error", "message": "Failed to fetch logs"}))
        }
    }

}
