use jsonwebtoken::{encode, Header, EncodingKey, errors::Error as JwtError};
use jsonwebtoken::{decode, DecodingKey, Validation};
use chrono::{Utc, Duration};
use std::env;

#[derive(serde::Serialize, serde::Deserialize)]
struct JWT {
    sub: String,
    roles: String,
    exp: usize,
    iat: usize
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
