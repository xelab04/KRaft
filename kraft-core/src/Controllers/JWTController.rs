use actix_web::cookie::Cookie;
use actix_web::cookie::time;
use actix_web::web;
use actix_web::{HttpRequest, cookie::SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, errors::ErrorKind};
use jsonwebtoken::{EncodingKey, Header, encode, errors::Error as JwtError};
use sqlx::PgPool;

use crate::Controllers::DBHelper::user;
use crate::Models::Config::AppConfig;
use crate::Models::JWT::{Claims, JWT};

pub async fn create_jwt(
    pool: &web::Data<PgPool>,
    app_config: &web::Data<AppConfig>,
    user_id: &str,
) -> String {
    let now = Utc::now();
    let expiration_time = now + Duration::hours(24);

    let exp_timestamp = expiration_time.timestamp() as usize;
    let iat_timestamp = now.timestamp() as usize;

    let int_user_id = user_id.parse::<i32>().unwrap();

    let role = if user::is_admin(pool, &int_user_id).await.unwrap_or(false) {
        "admin"
    } else {
        "base"
    };

    let payload = JWT {
        sub: user_id.to_string(),
        roles: role.to_string(),
        exp: exp_timestamp,
        iat: iat_timestamp,
    };

    let token = encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(app_config.jwt_secret.as_bytes()),
    );

    token.unwrap()
}

pub fn create_cookie(jwt_token: &str) -> Cookie<'_> {
    Cookie::build("auth_token", jwt_token.to_string())
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(1800))
        .finish()
}

pub fn del_cookie() -> Cookie<'static> {
    Cookie::build("auth_token", "")
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(0))
        .finish()
}

pub fn extract_user_id_from_jwt(req: &HttpRequest) -> Result<String, JwtError> {
    let app_config = req
        .app_data::<web::Data<AppConfig>>()
        .expect("failed to retrieve app config");

    let cookie_token = req
        .cookie("auth_token")
        .map(|cookie| cookie.value().to_string())
        .unwrap_or(String::from(""));

    if cookie_token.is_empty() {
        return Err(JwtError::from(ErrorKind::InvalidToken));
    }

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    // present by default, but whatever
    let token_data = decode::<Claims>(
        &cookie_token,
        &DecodingKey::from_secret(app_config.jwt_secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims.sub)
}

// pub fn validate_jwt(jwt_secret: String, jwt: &str) -> bool {
//     let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
//     let token_data = decode::<JWT>(
//         &jwt.to_string(),
//         &DecodingKey::from_secret(jwt_secret.as_bytes()),
//         &validation,
//     );
//     match token_data {
//         Ok(_) => {return true;}
//         Err(_) => {return false;}
//     }
// }
