use jsonwebtoken::{encode, Header, EncodingKey, errors::Error as JwtError};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use actix_web::HttpRequest;
use chrono::{Utc, Duration};
use std::env;

#[derive(serde::Serialize, serde::Deserialize)]
struct JWT {
    sub: String,
    roles: String,
    exp: usize,
    iat: usize
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_jwt(user_id: String) -> String {

    let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in environment variables");

    let now = Utc::now();
    let expiration_time = now + Duration::hours(24);

    let exp_timestamp = expiration_time.timestamp() as usize;
    let iat_timestamp = now.timestamp() as usize;

    let payload = JWT {
        sub: user_id,
        roles: "admin".to_string(),
        exp: exp_timestamp,
        iat: iat_timestamp
    };

    let token = encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    );

    token.unwrap()
}


pub fn validate_jwt(jwt: &String) -> bool {

    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in environment variables");

    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    let token_data = decode::<JWT>(
        &jwt,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    );

    match token_data {
        Ok(_) => {return true;}
        Err(_) => {return false;}
    }
}

pub fn extract_user_id_from_jwt(req: &HttpRequest) -> Result<String, JwtError> {
    let JWT_SECRET = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in environment variables");

    let cookie_token = req
        .cookie("auth_token")
        .map(|cookie| cookie.value().to_string())
        .unwrap_or(String::from(""));

    if cookie_token.is_empty() {
        return Err(JwtError::from(ErrorKind::InvalidToken));
    }

    let token_data = decode::<Claims>(
        &cookie_token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims.sub)
}
