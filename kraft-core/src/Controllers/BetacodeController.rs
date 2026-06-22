use rand::{distributions::Alphanumeric, Rng};
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

    let valid_betacode: Option<String> = sqlx::query_scalar("SELECT betacode FROM betacode WHERE enabled=true LIMIT 1")
        .fetch_optional(pool)
        .await?;
    let admin_user_exists: bool = sqlx::query_scalar("SELECT EXISTS ( SELECT 1 FROM users LIMIT 1 )")
        .fetch_one(pool)
        .await?;
    // TODO
    //
    // if there is something in the beta code table but there are no users...
    // do we generate yet another beta code? -> restart looping before registration will make MANY codes
    // --- do we output a funtional code? -> have to check and find a functional beta code, or generate one otherwise
    // do we check that at least one code must be valid at start? -> no, you might want registration to be closed

    // if there are no admin users, and no valid beta codes, generate one and output it
    if !admin_user_exists && valid_betacode.is_none() {
        info!("no admin user and no valid beta codes exists, generating one");
        let code = generate_code(pool).await.unwrap();
        output_code(&code);
        return Ok(());
    }

    // if there are no admin users, and a valid beta code, output it
    // this prevents code generation on crashloop for example
    if let Some(code) = valid_betacode {
        info!("no admin user but valid beta code exists, reusing it");
        output_code(&code);
        return Ok(());
    }

    async fn generate_code(pool: &PgPool) -> Result<String, sqlx::Error> {
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
        Ok(betacode_text)
    }

    fn output_code(betacode_text: &String) {
        info!("------------");
        info!("use the following registration code to create the first account");
        info!("this account will gain full admin privileges");
        info!("the registration code will be disabled after use");
        info!("registration code: {}", betacode_text);
        info!("------------");
    }

    Ok(())
}
