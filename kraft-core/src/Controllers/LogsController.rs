use actix_web::HttpResponse;
use actix_web::web;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx;
use sqlx::PgPool;

use k3k_rs;
use kube::Client;

use crate::Controllers::DBHelper::clusters;
use crate::Models::User::AuthUser;

#[derive(Serialize, Deserialize)]
pub struct LogsType {
    logtype: String,
    full_cluster_name: String,
}

#[get("/api/logs")]
pub async fn getlogs(
    pool: web::Data<PgPool>,
    client: web::Data<Client>,
    // config: web::Data<AppConfig>,
    query: web::Query<LogsType>,
    user: AuthUser,
) -> HttpResponse {
    let user_id = user.user_id;
    let cluster_name = &query.full_cluster_name;
    let namespace = format!("k3k-{}", cluster_name);
    let int_user_id = user_id.parse::<i32>().unwrap();

    let logtype = &query.logtype;

    if !clusters::name_belongs_to(&pool, &int_user_id, cluster_name)
        .await
        .unwrap()
    {
        return HttpResponse::NotFound()
            .json(json!({"status": "error", "message": "Cluster not found under this user"}));
    }

    // default to server
    let logs_returned = if logtype == "agent" {
        k3k_rs::logs::agent(&client, cluster_name, &namespace, 10).await
    } else {
        k3k_rs::logs::server(&client, cluster_name, &namespace, 10).await
    };

    match logs_returned {
        Ok(logs) => HttpResponse::Ok().json(json!({"status": "success", "logs": logs})),
        Err(e) => {
            error!("Failed to fetch logs: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Failed to fetch logs"}))
        }
    }
}
