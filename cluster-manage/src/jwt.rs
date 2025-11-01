use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, errors::Error as JwtError, errors::ErrorKind};
use actix_web::{HttpRequest};
use serde::{Deserialize, Serialize};
use reqwest;


// const JWT_SECRET: &str = "your-shared-secret"; // same as in Python service



#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn extract_user_id_from_jwt(req: &HttpRequest) -> Result<String, JwtError> {

    // let body = reqwest::get("https://www.rust-lang.org")
    //     .await?
    //     .text()
    //     .await?;


    let JWT_SECRET = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in environment variables");

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

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
