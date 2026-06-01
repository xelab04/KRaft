use argon2::password_hash::PasswordHash;
use argon2::{Argon2, PasswordVerifier};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, SameSite},
    web,
};
use log::{error, info};
use serde_json::{self, json};
use sqlx::PgPool;
use uuid;

// use crate::jwt;
// use crate::util::{check_passwords_match, hash_password};
// use crate::class::{AppConfig, AuthUser, Claims, PasswordChange, PasswordParams, User};
// use crate::util::send_mail;

use crate::Controllers::DBHelper::{password, user};
use crate::Controllers::{JWTController, utils};
use crate::Models::Config::AppConfig;
use crate::Models::JWT::Claims;
use crate::Models::Password::PasswordChange;
use crate::Models::User::{AuthUser, User};

// #[actix_web::get("/auth/password")]
// pub async fn generate_password(query: web::Query<PasswordParams>) -> HttpResponse {
//     let password = &query.user_password;
//     let hash = utils::hash_password(password);

//     HttpResponse::Ok().json(json!(
//         {
//             "password": hash
//         }
//     ))
// }

#[actix_web::post("/auth/changepassword")]
pub async fn changepwd(
    pool: web::Data<PgPool>,
    payload: web::Json<PasswordChange>,
    // req: HttpRequest,
    user: AuthUser,
) -> HttpResponse {
    let user_id = user.user_id;

    let user_password: String =
        sqlx::query_scalar("SELECT password FROM users WHERE user_id = ($1)")
            .bind(&user_id)
            .fetch_one(pool.get_ref())
            .await
            .unwrap();

    if utils::check_passwords_match(&payload.current_password, &user_password) {
        let new_hashed_password = utils::hash_password(&payload.new_password);

        let int_user_id = user_id.parse::<i32>().unwrap();
        password::update(&pool, &new_hashed_password, &int_user_id)
            .await
            .expect("error updating password");
        info!("user {} changed password", user_id);
        HttpResponse::Ok().json(json!({ "status": "success" }))
    } else {
        HttpResponse::Forbidden().json(json!({"message": "Invalid password"}))
    }
}

#[actix_web::post("/auth/logout")]
pub async fn logout() -> HttpResponse {
    let cookie = JWTController::del_cookie();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({ "status": "success", "message": "success" }))
}

#[actix_web::post("/auth/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    app_config: web::Data<AppConfig>,
    payload: web::Json<User>,
) -> HttpResponse {
    let email = &payload.email;
    // let user_id = &payload.user_id;
    // let int_user_id = user_id.unwrap();
    let user_password = &payload.user_password;

    if email.is_empty() || user_password.is_empty() {
        return HttpResponse::Unauthorized().finish();
    }

    let fetch_user = user::get_details_from_email(&pool, email).await;
    let found_user = match fetch_user {
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(json!({ "status": "failure", "message": "Incorrect email/password" }));
        }
        Ok(user) => user,
    };

    let parsed_hash = match PasswordHash::new(&found_user.user_password) {
        Ok(hash) => hash,
        Err(e) => {
            error!(
                "Error parsing password hash from DB for user {}: {:?}",
                found_user.email, e
            );
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "false",
                "message": "An internal error occured"
            }));
        }
    };

    let jwt_token = JWTController::create_jwt(
        &pool,
        &app_config,
        &found_user
            .user_id
            .expect("Attempted to find user id in db")
            .to_string(),
    )
    .await;

    match Argon2::default().verify_password(user_password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            let cookie = JWTController::create_cookie(&jwt_token);

            let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());
            if environment == "PROD" {
                return HttpResponse::Ok().cookie(cookie).json(
                    json!({ "status": "success", "message": "success", "uuid": &found_user.uuid }),
                );
            }

            info!("user {} logged in", found_user.user_id.unwrap());
            HttpResponse::Ok()
                .cookie(cookie)
                .json(json!({ "status": "success", "uuid": &found_user.uuid }))
        }
        Err(_) => HttpResponse::Forbidden()
            .json(json!({ "status": "failure", "message": "Incorrect email/password" })),
    }
}

#[actix_web::post("/auth/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    app_config: web::Data<AppConfig>,
    payload: web::Json<User>,
) -> HttpResponse {
    let user = &payload.username;
    let email = &payload.email;
    let user_password = &payload.user_password;
    let default = &"".to_string();
    let betacode = &payload.betacode.as_ref().map_or(default, |s| s); //.as_ref().map_or("", |s| s.as_str());

    let betacode_enabled: bool = !std::env::var("BETACODE")
        .unwrap_or("".to_string())
        .is_empty();

    if betacode_enabled {
        let all_beta_codes: Vec<String> =
            sqlx::query_scalar("SELECT betacode FROM betacode WHERE enabled = TRUE")
                .fetch_all(pool.get_ref())
                .await
                .unwrap();
        if !all_beta_codes.contains(betacode) {
            return HttpResponse::Forbidden()
                .json(json!({ "status": "error", "message": "Invalid beta code" }));
        }
    }

    let same_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ($1)")
        .bind(email)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    if same_users > 0 {
        return HttpResponse::Conflict()
            .json(json!({ "status": "error", "message": "User already exists with that email" }));
    }

    let username = user.clone().unwrap();
    let same_username = user::same_username(&pool, &username).await.unwrap();
    if same_username {
        return HttpResponse::Conflict().json(
            json!({ "status": "error", "message": "User already exists with that username" }),
        );
    }

    // alex you idiot, you forgot to hash the password TwT
    let password_hash = utils::hash_password(&user_password.to_string());
    let user_uuid = uuid::Uuid::new_v4().to_string();
    let email_validation = uuid::Uuid::new_v4().to_string();

    let user_id = sqlx::query_scalar::<_, i32>("INSERT INTO users (username, email, password, betacode, uuid, verification_code, admin, verified_email) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING user_id")
        .bind(user)
        .bind(email)
        .bind(password_hash)
        .bind(betacode)
        .bind(user_uuid)
        .bind(&email_validation)
        .bind(false)
        .bind(false)
        .fetch_one(pool.get_ref())
        .await;

    match user_id {
        Ok(uid) => {
            let user_id: i32 = uid;
            // if user created succesfully, generate cookie
            // let user_id = pg_result.last_insert_id();

            let jwt_token =
                JWTController::create_jwt(&pool, &app_config, &user_id.to_string()).await;

            let cookie = Cookie::build("auth_token", &jwt_token)
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .finish();

            // if we have a mail config and mail_verification is active, send a verification mail
            if app_config.mail_verification {
                if let Some(mail_config) = &app_config.email {
                    let subject = "Confirm your email for KRaft";

                    let validation_link =
                        format!("{}/auth/validate/{}", app_config.host, email_validation);
                    let body = format!("Thank you for creating an account on KRaft, please confirm your email address using the following link:
                        \n{validation_link}");
                    utils::send_mail(mail_config, email, subject, body.as_str())
                        .await
                        .unwrap();
                } else {
                    error!("Mail verification enabled but no valid mail config set");
                }
            }

            info!("new account for user {}", user_id);
            HttpResponse::Ok()
                .cookie(cookie)
                .json(json!({ "status": "success" }))
        }
        Err(e) => {
            error!("Error inserting user: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::get("/auth/validate-jwt")]
pub async fn validate_jwt(req: HttpRequest, app_config: web::Data<AppConfig>) -> HttpResponse {
    // let auth_header = req
    //     .headers()
    //     .get("Authorization")
    //     .and_then(|h| h.to_str().ok())
    //     .unwrap_or("");

    let cookie_token = req
        .cookie("auth_token")
        .map(|cookie| cookie.value().to_string())
        .unwrap_or(String::from(""));

    if cookie_token.is_empty() {
        return HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Token not found.",
        }));
    }

    let res = decode::<Claims>(
        &cookie_token,
        &DecodingKey::from_secret(app_config.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    );

    match res {
        Ok(data) => {
            let token_data = data.claims;
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": format!("User ID: {}", token_data.sub),
            }))
        }
        Err(_) => HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid token."
        })),
    }
}
