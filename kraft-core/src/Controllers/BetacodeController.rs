use log::{error, info};
use std::collections::BTreeMap;

use actix_web::web;
use actix_web::web::Json;
use actix_web::{HttpRequest, HttpResponse};

use sqlx;
use sqlx::PgPool;

use crate::Controllers::DBHelper::*;
use crate::Models::Betacode::Betacode;
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

#[put("/api/admin/betacode/update")]
pub async fn update(
    _req: HttpRequest,
    pool: web::Data<PgPool>,
    user: AuthUser,
    Json(betacode): Json<Betacode>,
) -> HttpResponse {
    let user_id: i32 = user.user_id.parse().unwrap();
    if !user::is_admin(&pool, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden().finish();
    }

    match betacode::update(&pool, &betacode).await {
        Ok(_) => {
            return HttpResponse::Ok().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(e.to_string());
        }
    }
}

#[post("/api/admin/betacode/new")]
pub async fn new(
    _req: HttpRequest,
    pool: web::Data<PgPool>,
    user: AuthUser,
    Json(betacode): Json<Betacode>,
) -> HttpResponse {
    let user_id: i32 = user.user_id.parse().unwrap();
    if !user::is_admin(&pool, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden().finish();
    }

    match betacode::create(&pool, &betacode).await {
        Ok(_) => {
            return HttpResponse::Ok().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(e.to_string());
        }
    }
}

#[post("/api/admin/betacode/delete")]
pub async fn delete(
    _req: HttpRequest,
    pool: web::Data<PgPool>,
    user: AuthUser,
    Json(betacode): Json<Betacode>,
) -> HttpResponse {
    let user_id: i32 = user.user_id.parse().unwrap();
    if !user::is_admin(&pool, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden().finish();
    }

    match betacode::delete(&pool, &betacode).await {
        Ok(_) => {
            return HttpResponse::Ok().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(e.to_string());
        }
    }
}

pub async fn first_startup(pool: &PgPool) -> Result<(), sqlx::Error> {

    // if a beta code exists, it means the initial admin *must* have been created already
    let betacode_exists: bool = sqlx::query_scalar("SELECT EXISTS ( SELECT betacode FROM betacode LIMIT 1 )")
        .fetch_one(pool)
        .await?;
    let user_exists: bool = sqlx::query_scalar("SELECT EXISTS ( SELECT 1 FROM users LIMIT 1 )")
        .fetch_one(pool)
        .await?;
    if betacode_exists || user_exists {
        info!("a beta code or user already exists, skipping first-user preparation");
    }

    use rand::{distributions::Alphanumeric, Rng};

    let betacode_text: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let _r = sqlx::query("INSERT INTO betacode (betacode, enabled) VALUES ($1, $2)")
        .bind(&betacode_text)
        .bind(true)
        .execute(pool)
        .await?;

    info!("------------");
    info!("use the following registration code to create the first account");
    info!("this account will gain full admin privileges");
    info!("the registration code will be disabled after use");
    info!("registration code: {}", betacode_text);
    info!("------------");

    Ok(())
}
