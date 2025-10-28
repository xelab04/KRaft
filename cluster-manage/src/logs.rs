use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx;
use sqlx::MySqlPool;

use k3k_rs;
use kube::Client;

use crate::jwt;
use crate::AppConfig;

#[derive(Serialize, Deserialize)]
pub struct LogsType {
    logtype: String,
    full_cluster_name: String
}

#[get("/api/logs")]
pub async fn getlogs(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    query: web::Query<LogsType>
) -> HttpResponse {

    let jwt = jwt::extract_user_id_from_jwt(&req);
    // let cluster_name = format!("{}-{}", user_id, &query.name);
    let cluster_name = &query.full_cluster_name;
    let namespace = format!("k3k-{}", cluster_name);

    let logtype = &query.logtype;

    let client = Client::try_default().await.unwrap();

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
    let cluster_id_from_db: Result<i64, sqlx::Error> = sqlx::query_scalar("SELECT cluster_id FROM clusters WHERE user_id = ? AND cluster_name = ?")
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(pool.get_ref())
        .await;

    let mut cluster_id;
    match cluster_id_from_db {
        Ok(id) => {
            cluster_id = id;
        }
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().json(json!({"status": "error", "message": "Cluster not found"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({"status": "error", "message": format!("Failed to check cluster: {}", e)}));
        }
    }

    // default to server
    let logs_returned;
    if logtype == "agent" {
        logs_returned = k3k_rs::logs::agent(&client, &cluster_name, &namespace, 10).await;
    } else {
        logs_returned = k3k_rs::logs::server(&client, &cluster_name, &namespace, 50).await;
    }

    match logs_returned {
        Ok(logs) => {
            HttpResponse::Ok().json(json!({"status": "success", "logs": logs}))
        }
        Err(e) => {
            println!("Failed to fetch logs: {}", e);
            HttpResponse::InternalServerError().json(json!({"status": "error", "message": "Failed to fetch logs"}))
        }
    }

}
