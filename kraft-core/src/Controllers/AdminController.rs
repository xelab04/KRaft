use log::{error, info};
use std::collections::BTreeMap;

use actix_web::web;
use actix_web::web::Json;
use actix_web::{HttpRequest, HttpResponse};

use sqlx;
use sqlx::PgPool;

use crate::Controllers::DBHelper::*;
use crate::Models::Config::AppConfig;

use crate::Models::User::AuthUser;

#[get("/api/admin/betacode/list")]
pub async fn create(_req: HttpRequest, pool: web::Data<PgPool>, user: AuthUser) -> HttpResponse {
    let user_id: i32 = user.user_id.parse().unwrap();
    if !user::is_admin(&pool, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden().finish();
    }

    let betacodes = betacode::list(&pool)
        .await
        .expect("Error retrieving betacodes from db");

    HttpResponse::Ok().json(betacodes)
}
