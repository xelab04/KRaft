use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx;
use sqlx::PgPool;

use k3k_rs;
use kube::Client;

use crate::class::AuthUser;
use crate::jwt;
use crate::AppConfig;

#[derive(Serialize, Deserialize)]
pub struct LogsType {
    logtype: String,
    full_cluster_name: String
}

#[get("/api/logs")]
pub async fn getlogs(
    // req: HttpRequest,
    pool: web::Data<PgPool>,
    client: web::Data<Client>,
    // config: web::Data<AppConfig>,
    query: web::Query<LogsType>,
    user: AuthUser
) -> HttpResponse {

    let user_id = user.user_id;
    let cluster_name = &query.full_cluster_name;
    let namespace = format!("k3k-{}", cluster_name);

    let logtype = &query.logtype;

    // check user owns that cluster
    let user_owns_cluster: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = $1 AND cluster_name = $2)")
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(false);

    if !user_owns_cluster {
        return HttpResponse::NotFound().json(json!({"status": "error", "message": "Cluster not found under this user"}));
    }

    // let cluster_id_from_db: Result<i64, sqlx::Error> = sqlx::query_scalar("SELECT cluster_id FROM clusters WHERE user_id = ? AND cluster_name = ?")
    //     .bind(user_id)
    //     .bind(cluster_name)
    //     .fetch_one(pool.get_ref())
    //     .await;

    // let mut cluster_id;
    // match cluster_id_from_db {
    //     Ok(id) => {
    //         cluster_id = id;
    //     }
    //     Err(sqlx::Error::RowNotFound) => {
    //         return HttpResponse::NotFound().json(json!({"status": "error", "message": "Cluster not found"}));
    //     }
    //     Err(e) => {
    //         return HttpResponse::InternalServerError().json(json!({"status": "error", "message": format!("Failed to check cluster: {}", e)}));
    //     }
    // }

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
