use actix_web::mime::JSON;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::Salt};
use argon2::{password_hash::{PasswordHash, SaltString, Error}};
use actix_web::{web, HttpRequest, HttpResponse, http::header, cookie::Cookie, cookie::SameSite};
use rand;
// use actix_web::web::{Json, Path};
use serde::{Serialize, Deserialize};
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::FromRow;
use serde_json;
use log::{info};

use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

use crate::jwt;

#[derive(serde::Serialize, serde::Deserialize, Debug, FromRow, Clone)]
struct User {
    user_id: Option<i32>,
    username: Option<String>,
    email: String,
    #[serde(rename = "password")]
    user_password: String,
    #[sqlx(skip)]
    betacode: Option<String>
}

#[derive(Deserialize)]
struct PasswordParams{
    user_password: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[actix_web::get("/auth/password")]
pub async fn password(query: web::Query<PasswordParams>) -> HttpResponse {

    let password = &query.user_password;

    let salt_str = &SaltString::generate(&mut rand::rngs::OsRng);
    let salt: Salt = salt_str.try_into().unwrap();

    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), salt).unwrap();

    return HttpResponse::Ok().json(json!(
        {
            "password": hash.to_string()
        }
    ))

}

#[actix_web::post("/auth/login")]
pub async fn login(pool: web::Data<MySqlPool>, payload: web::Json<User>) -> HttpResponse {

    let email = &payload.email;
    let user_password = &payload.user_password;

    if email == "" || user_password == "" {
        return HttpResponse::Unauthorized().finish();
    }

    let user_data = sqlx::query_as::<_, User>(
        "SELECT user_id, username, email, password as user_password FROM users WHERE email = (?)"
        )
        .bind(email)
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    if user_data.len() == 0 {
        return HttpResponse::Unauthorized().finish();
    }

    let found_user = &user_data[0];

    let parsed_hash = match PasswordHash::new(&found_user.user_password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Error parsing password hash from DB for user {}: {:?}", found_user.email, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "false",
                "message": "An internal error occured"
            }))
        }
    };

    let jwt_token = jwt::create_jwt(found_user.user_id.expect("Attempted to find user id in db").to_string());

    match Argon2::default().verify_password(user_password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            let cookie = Cookie::build("auth_token", &jwt_token)
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .finish();

            let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "prod".to_string());
            if environment == "prod" {
                return HttpResponse::Ok()
                    .cookie(cookie)
                    .json(json!({ "status": "success", "message": "success" }))
            }

            HttpResponse::Ok()
                .cookie(cookie)
                .json(json!({ "status": "success" }))

        }
        Err(_) => {return HttpResponse::Forbidden().finish();}
    }

}

#[actix_web::post("/auth/register")]
pub async fn register(pool: web::Data<MySqlPool>, payload: web::Json<User>) -> HttpResponse {
    let user = &payload.username;
    let email = &payload.email;
    let user_password = &payload.user_password;
    let betacode = &payload.betacode.as_ref().map_or("", |s| s.as_str());


    let valid_beta_code = std::env::var("BETACODE").unwrap_or("".to_string());

    // if beta code is not valid
    // and actual beta code is not empty
    if *betacode != valid_beta_code.as_str() && valid_beta_code != "" {
        return HttpResponse::Forbidden().json(json!({ "status": "error", "message": "Invalid beta code" }));
    }

    let same_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = (?)")
        .bind(email)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    if same_users > 0 {
        return HttpResponse::Conflict().json(json!({ "status": "error", "message": "User already exists with that email" }));
    }

    let same_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = (?)")
        .bind(user)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    if same_users > 0 {
        return HttpResponse::Conflict().json(json!({ "status": "error", "message": "User already exists with that username" }));
    }


    // alex you idiot, you forgot to hash the password TwT

    let salt_str = &SaltString::generate(&mut rand::rngs::OsRng);
    let salt: Salt = salt_str.try_into().unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(user_password.as_bytes(), salt).unwrap();

    let r = sqlx::query("INSERT INTO users (username, email, password) VALUES (?, ?, ?)")
        .bind(user)
        .bind(email)
        .bind(password_hash.to_string())
        .execute(pool.get_ref())
        .await;

    // In the future, have email verification

    match r {
        Ok(_) => {return HttpResponse::Ok().json(json!({ "status": "success" }))}
        Err(e) => {
            println!("Error inserting user: {}", e);
            return HttpResponse::InternalServerError().finish();}
    }
}


#[actix_web::get("/auth/validate-jwt")]
pub async fn validate_jwt(req: HttpRequest) -> HttpResponse {

    let JWT_SECRET = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in environment variables");

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
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    );

    match res {
        Ok(data) => {
            let token_data = data.claims;
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "message": format!("User ID: {}", token_data.sub),
            }));
        }
        Err(_) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "Invalid token."
            }));
        }
    }

}
