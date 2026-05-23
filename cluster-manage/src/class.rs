use actix_web::{Error, FromRequest, HttpRequest, web};
use futures_util::future::{ready, Ready};
use reqwest;
use log::{info};
use regex::Regex;

use sqlx::{FromRow, PgPool};
use serde::{self, Serialize, Deserialize};
use crate::{AppConfig, NtfyConfig, jwt};

use kube::{
    api::{Api, PostParams},
    core::{DynamicObject, GroupVersionKind, ApiResource},
    Client,
};
use serde_json::json;


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

pub fn namevalid(name: &String) -> bool {

    return name.chars().all(|ch|
        (ch >= 'a' && ch <= 'z')
        || (ch >= '0' && ch <= '9')
        || ch == '-'
    );
}

pub async fn traefik(client: &Client, cluster_name: &String, namespace: &String, host: &str, n: usize) -> bool {

    // define CRD type
    let gvk = GroupVersionKind::gvk("traefik.io", "v1alpha1", "IngressRouteTCP");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let ingress_routes: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace.as_str(), &ar);

    // json cause im lazy
    let ingressroute = json!({
        "apiVersion": "traefik.io/v1alpha1",
        "kind": "IngressRouteTCP",
        "metadata": {
            "name": format!("api-svr-{}-{}-rt",cluster_name,n),
            "namespace": namespace
        },
        "spec": {
            "entryPoints": ["websecure"],
            "routes": [
                {
                    "match": format!("HostSNI(`{}`)", host),
                    "services": [
                        {
                            "name": format!("k3k-{}-service",cluster_name),
                            "port": 443
                        }
                    ]
                }
            ],
            "tls": { "passthrough": true }
        }
    });

    let pp = PostParams::default();
    let ingressroute: DynamicObject = serde_json::from_value(ingressroute).unwrap();

    let _created = ingress_routes.create(&pp, &ingressroute).await.unwrap();

    true
}

pub async fn validate_tlssan(tlssan: String) -> Result<bool, String> {
    if !tlssan.is_ascii() {
        return Err("Invalid URL".to_string());
    }

    let domain_pattern = r"^([A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.)+[A-Za-z]{2,63}$";
    let re = Regex::new(domain_pattern).unwrap();

    if !re.is_match(&tlssan) {
        return Err("Malformed URL".to_string());
    }

    return Ok(true);
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
