use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, errors::Error as JwtError, errors::ErrorKind};
use actix_web::{HttpRequest};
use serde::{Deserialize, Serialize};


const JWT_SECRET: &str = "your-shared-secret"; // same as in Python service

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    // Python JWT structure
}

pub fn extract_user_id_from_jwt(req: &HttpRequest) -> Result<String, JwtError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return Err(JwtError::from(ErrorKind::InvalidToken));
    }

    let token = &auth_header[7..]; // strip "Bearer "
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims.sub)
}
