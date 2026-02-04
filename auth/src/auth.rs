use argon2::{Argon2, PasswordVerifier};
use argon2::{password_hash::{PasswordHash}};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

use actix_web::{web, HttpRequest, HttpResponse, cookie::{Cookie, SameSite}};
use uuid;
use serde_json::{self, json};
use sqlx::{MySqlPool};

use crate::jwt;
use crate::util::{check_passwords_match, hash_password};
use crate::class::{AppConfig, AuthUser, Claims, PasswordChange, PasswordParams, User};
use crate::util::send_mail;

#[actix_web::get("/auth/password")]
pub async fn password(query: web::Query<PasswordParams>) -> HttpResponse {

    let password = &query.user_password;
    let hash = hash_password(password);

    return HttpResponse::Ok().json(json!(
        {
            "password": hash
        }
    ))
}

#[actix_web::post("/auth/changepassword")]
pub async fn changepwd(
    pool: web::Data<MySqlPool>,
    payload: web::Json<PasswordChange>,
    // req: HttpRequest,
    user: AuthUser
) -> HttpResponse {

    let user_id = user.user_id;

    let user_password:String = sqlx::query_scalar("SELECT password FROM users WHERE user_id = (?)")
        .bind(&user_id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();

    if check_passwords_match(&payload.current_password, &user_password) {

        let new_hashed_password = hash_password(&payload.new_password);

        sqlx::query("UPDATE users SET password = (?) WHERE user_id = (?)")
            .bind(new_hashed_password)
            .bind(&user_id)
            .execute(pool.get_ref())
            .await
            .unwrap();

        HttpResponse::Ok()
            .json(json!({ "status": "success" }))
    } else {
        return HttpResponse::Forbidden().json(json!({"message": "Invalid password"}));
    }
}


#[actix_web::post("/auth/logout")]
pub async fn logout() -> HttpResponse {
    let cookie = jwt::del_cookie();

    return HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({ "status": "success", "message": "success" }));
}

#[actix_web::post("/auth/login")]
pub async fn login(pool: web::Data<MySqlPool>, payload: web::Json<User>) -> HttpResponse {

    let email = &payload.email;
    let user_password = &payload.user_password;

    if email == "" || user_password == "" {
        return HttpResponse::Unauthorized().finish();
    }

    let user_data = sqlx::query_as::<_, User>(
        "SELECT user_id, username, email, password as user_password, betacode, uuid FROM users WHERE email = (?)"
        )
        .bind(email)
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    if user_data.len() == 0 {
        return HttpResponse::Unauthorized().json(json!({ "status": "failure", "message": "Incorrect email/password" }));
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
            let cookie = jwt::create_cookie(&jwt_token);

            let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());
            if environment == "PROD" {
                return HttpResponse::Ok()
                    .cookie(cookie)
                    .json(json!({ "status": "success", "message": "success", "uuid": &found_user.uuid }))
            }

            HttpResponse::Ok()
                .cookie(cookie)
                .json(json!({ "status": "success", "uuid": &found_user.uuid }))

        }
        Err(_) => {return HttpResponse::Forbidden().json(json!({ "status": "failure", "message": "Incorrect email/password" }));}
    }

}

#[actix_web::post("/auth/register")]
pub async fn register(
    pool: web::Data<MySqlPool>,
    app_config: web::Data<AppConfig>,
    payload: web::Json<User>
) -> HttpResponse {
    let user = &payload.username;
    let email = &payload.email;
    let user_password = &payload.user_password;
    let default = &"".to_string();
    let betacode = &payload.betacode.as_ref().map_or(default, |s| s); //.as_ref().map_or("", |s| s.as_str());

    let betacode_enabled: bool = !std::env::var("BETACODE").unwrap_or("".to_string()).is_empty();

    if betacode_enabled {
        let all_beta_codes: Vec<String> = sqlx::query_scalar("SELECT betacode FROM betacode WHERE enabled = TRUE")
            .fetch_all(pool.get_ref())
            .await
            .unwrap();
        if ! all_beta_codes.contains(betacode) {
            return HttpResponse::Forbidden().json(json!({ "status": "error", "message": "Invalid beta code" }));
        }
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
    let password_hash = hash_password(&user_password.to_string());
    let user_uuid = uuid::Uuid::new_v4().to_string();
    let email_validation = uuid::Uuid::new_v4().to_string();

    let r = sqlx::query("INSERT INTO users (username, email, password, betacode, uuid, verification_code, admin) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(user)
        .bind(email)
        .bind(password_hash)
        .bind(betacode)
        .bind(user_uuid)
        .bind(&email_validation)
        .bind(false)
        .execute(pool.get_ref())
        .await;

    // In the future, have email verification

    match r {
        Ok(mysql_result) => {
            // if user created succesfully, generate cookie
            // let user_id = mysql_result.last_insert_id();

            let user_id: i64 = sqlx::query_scalar("SELECT user_id FROM users WHERE username = ?")
                .bind(user)
                .fetch_one(pool.get_ref())
                .await
                .unwrap();

            let jwt_token = jwt::create_jwt(user_id.to_string());

            let cookie = Cookie::build("auth_token", &jwt_token)
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .finish();

            if let Some(mail_config) = &app_config.email {
                let subject = "Confirm your email for KRaft";

                let validation_link = format!("{}/auth/validate/{}", app_config.host, email_validation);
                let body = format!("Thank you for creating an account on KRaft, please confirm your email address using the following link:
                    \n{validation_link}");
                let r = send_mail(&mail_config, email, subject, body.as_str())
                    .await
                    .unwrap();
            }

            return HttpResponse::Ok().cookie(cookie).json(json!({ "status": "success" }));
        }
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
