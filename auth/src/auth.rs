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

use crate::jwt;

#[derive(serde::Serialize, serde::Deserialize, Debug, FromRow, Clone)]
struct User {
    id: Option<i32>,
    email: String,
    #[serde(rename = "password")]
    user_password: String
}

#[derive(Deserialize)]
struct PasswordParams{
    user_password: String
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
        "SELECT user_id, email, password as user_password FROM users WHERE email = (?)"
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

    let jwt_token = jwt::create_jwt(found_user.id.expect("Attempted to find user id in db").to_string());

    match Argon2::default().verify_password(user_password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            let cookie = Cookie::build("auth_token", &jwt_token)
                .path("/admin")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .finish();

            HttpResponse::Ok()
                .cookie(cookie)
                .json(json!({ "status": "success" }))

            // return HttpResponse::Ok().json(json!({"status":"success", "jwt":jwt_token}));

        }
        Err(_) => {return HttpResponse::Forbidden().finish();}
    }

}


#[actix_web::get("/auth/validate-jwt")]
pub async fn validate_jwt(req: HttpRequest) -> HttpResponse {


    let cookie_auth_token: Option<String> = req
        .cookie("auth_token") // Get the cookie by name
        .map(|cookie| cookie.value().to_string()); // Extract its value as a String

    let jwt_token = if let Some(token) = cookie_auth_token {
        token
    } else {
        return HttpResponse::Unauthorized().finish();
    };

    if jwt::validate_jwt(&jwt_token) {
        return HttpResponse::Ok().json(json!({
            "success": true,
            "message": format!("JWT found and extracted: {}", jwt_token),
        }))
    }

    return HttpResponse::Unauthorized().json(json!({
        "success": false,
        "message": "Authorization header missing or malformed. Expected 'Bearer <token>'.".to_string(),
    }));
}
