use std::fmt::Error;

use actix_web::{HttpResponse, web::{self, Path}};
use reqwest::Response;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    Controllers::DBHelper::{towonel_db, user}, Models::{Config::{AppConfig, TowonelConfig}, User::{AuthUser, User}},
};

pub fn generate_url(hub: &str, request_path: &str) -> String {
    let mut host = hub.to_string();
    if !host.starts_with("https://") {
        host = format!("https://{host}")
    }
    if let Some(h) = host.strip_suffix("/") {
        host = String::from(h);
    }
    host = format!("{host}{request_path}");

    host
}

pub async fn existing_token(pool: &web::Data<PgPool>, user: &User, domain: &str, towonel_config: &TowonelConfig, token_id: &str) -> Response {
    let request_body = json!({"hostnames": Vec::from([domain]), "name": &user.username});
    let request_path = format!("/v1/invites/{token_id}/hostnames");
    let host = generate_url(&towonel_config.hub, &request_path);

    let result = send_towonel_request(&host, &towonel_config.token, request_body.to_string()).await;

    return result;
}

pub async fn new_token(pool: &web::Data<PgPool>, user: &User, domain: &str, towonel_config: &TowonelConfig) -> Response {
    let request_body = json!({"hostnames": Vec::from([domain])});
    let request_path = format!("/v1/invites");
    let host = generate_url(&towonel_config.hub, &request_path);

    let result = send_towonel_request(&host, &towonel_config.token, request_body.to_string()).await;

    return result;
}

pub async fn send_towonel_request(host: &str, token: &str, request_body: String) -> Response {
    let client = reqwest::Client::new();
    let response = client
        .post(host)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(request_body.to_string())
        .send()
        .await
        .unwrap();

    response
}

#[post("/api/towonel/domain/{domain}")]
pub async fn new_domain(
    domain: Path<String>,
    user: AuthUser,
    config: web::Data<AppConfig>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // get the other stuff... we call this legacy nonsense
    let int_user_id: i32 = user.user_id.parse().unwrap();
    let domain = domain.into_inner();
    let req_user = user::get_details(&pool, &int_user_id).await.unwrap();

    // check towonel is configured
    let towonel_config;
    if let Some(cfg) = &config.towonel_config {
        towonel_config = cfg;
    } else {
        return HttpResponse::NotImplemented().finish();
    }

    // check if user already has a token
    let token_id_opt = towonel_db::get_token_id_from_user_id(&int_user_id, &pool)
        .await
        .unwrap();

    let response: Response;
    if let Some(token_id) = token_id_opt {
        // token already exists, update it with new hostname
        response = existing_token(&pool, &req_user, &domain, towonel_config, &token_id).await;
    } else {
        // token doesn't exist, create a new one
        response = new_token(&pool, &req_user, &domain, towonel_config).await;
    }

    let status = response.status();
    let _response_text = response.text().await.unwrap();

    if status.is_success() {
        return HttpResponse::Ok().finish();
    } else {
        let code = status.as_u16();
        if code == 409 { return HttpResponse::Conflict().finish(); }
        return HttpResponse::InternalServerError().finish();
    }
}

#[get("/api/towonel/isactive")]
pub async fn is_towonel_configured(config: web::Data<AppConfig>) -> HttpResponse {
    if config.towonel_config.is_none() {
        return HttpResponse::NotImplemented().finish();
    }
    return HttpResponse::Ok().finish();
}
