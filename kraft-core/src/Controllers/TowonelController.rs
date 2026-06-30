use std::fmt::Error;

use actix_web::web;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    Controllers::DBHelper::{towonel_db, user},
    Models::{Config::TowonelConfig, User::AuthUser},
};

mod localutils {
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
}

pub struct ApiError(u16);
pub async fn new_domain(
    domain: &str,
    user: AuthUser,
    config: TowonelConfig,
    pool: web::Data<PgPool>,
) -> Result<(), ApiError> {
    let request_body = json!({"hostnames": Vec::from([domain])});
    let request_path: String;

    // get the other stuff... we call this legacy nonsense
    let int_user_id: i32 = user.user_id.parse().unwrap();
    let user_uuid = user::get_uuid_from_id(&pool, &int_user_id).await.unwrap();

    // check if user already has a token
    let token_id_opt = towonel_db::get_token_id_from_user_uuid(&user_uuid, &pool)
        .await
        .unwrap();

    if let Some(token_id) = token_id_opt {
        // token already exists, update it with new hostname
        request_path = format!("/v1/invites/{token_id}/hostnames");
    } else {
        // token doesn't exist, create a new one
        request_path = String::from("/v1/invites");
    }

    let host = localutils::generate_url(&config.hub, &request_path);

    let client = reqwest::Client::new();
    let response = client
        .post(host)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.token))
        .body(request_body.to_string())
        .send()
        .await
        .unwrap();

    let status = response.status();
    let _response_text = response.text().await.unwrap();

    if status.is_success() {
        return Ok(());
    } else {
        return Err(ApiError(status.as_u16()));
    }
}
