use actix_web::{Error, FromRequest, HttpRequest, web};
use futures_util::future::{ready, Ready};
use reqwest;
use log::{info};

use sqlx::{FromRow, PgPool};
use serde::{self, Serialize, Deserialize};
use crate::{AppConfig, NtfyConfig, jwt};


pub struct AuthUser {
    pub user_id: String
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        // if not running in prod, bypass auth
        let config = req.app_data::<web::Data<AppConfig>>();
        if let Some(cfg) = config {
            if cfg.environment != "PROD" {
                return ready(Ok(AuthUser { user_id: String::from("0") }))
            }
        }

        let jwt = jwt::extract_user_id_from_jwt(&req);
        match jwt {
            Ok(id) => { return ready(Ok(AuthUser { user_id: id })); }
            Err(_) => { return ready(Err(actix_web::error::ErrorUnauthorized("Unauthorised"))); }
        };
    }
}

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

pub fn panic_ntfy(config: &NtfyConfig, message: &str, title: &str) {
    let client = reqwest::blocking::Client::new();

    let mut request = client.post(&config.host)
        .header("Title", title)
        .body(message.to_string());

    if let Some(auth) = &config.basic_auth { request = request.header("Authorization", format!("Basic {auth}")); }
    if let Some(auth) = &config.token { request = request.header("Authorization", format!("Bearer {auth}")); }

    match request.send() {
        Ok(r) => {

            match r.error_for_status() {
                Ok(_) => { info!("Ntfy panic message sent"); }
                Err(e) => { info!("Error message; {}", e); }
            }
        }
        Err(_) => { info!("Error sending ntfy panic message, ironic") }
    }
}

pub async fn send_ntfy_notif(host: &str, message: &str, title: &str, basic_auth: &Option<String>, token: &Option<String>) -> Result<(), String> {
    let client = reqwest::Client::new();
    let mut request = client.post(host)
        .header("Title", title)
        .body(message.to_string());

    if let Some(auth) = basic_auth {
        request = request.header("Authorization", format!("Basic {auth}"));
    }
    if let Some(auth) = token {
        request = request.header("Authorization", format!("Bearer {auth}"));
    }

    let r = request.send()
        .await
        .unwrap();

    println!("{:?}", r);

    match r.error_for_status() {
        Ok(_) => { return Ok(()); }
        Err(e) => { return Err(e.to_string()); }
    }
}

pub async fn check_cluster_ownership(pool: &web::Data<PgPool>, user_id: &i32, cluster_name: Option<&String>, cluster_id: Option<&i32>) -> bool {
    let cluster_belongs_to_user: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = $1 AND (cluster_name = $2 OR cluster_id = $3))")
        .bind(&user_id)
        .bind(&cluster_name)
        .bind(&cluster_id)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to fetch cluster count");

    return cluster_belongs_to_user
}

pub async fn get_cluster_id_from_name(pool: &web::Data<PgPool>, user_id: &i32, cluster_name: &str) -> i32 {
    let int_cluster_id = sqlx::query_scalar("SELECT cluster_id FROM clusters WHERE user_id = $1 AND cluster_name = $2")
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to get cluster id");

    return int_cluster_id;
}
