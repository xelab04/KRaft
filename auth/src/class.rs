use futures_util::future::{ready, Ready};
use actix_web::{HttpRequest, FromRequest, Error};
use serde::{self, Serialize, Deserialize};
use sqlx::FromRow;

use crate::jwt;

pub struct MailConfig {
    pub mail_encryption: String,
    pub mail_from_address: String,
    pub mail_from_name: String,
    pub mail_host: String,
    pub mail_mailer: String,
    pub mail_port: String,
    pub mail_password: Option<String>,
    pub mail_username: Option<String>,
}

// User ID from Request
pub struct AuthUser {
    pub user_id: String
}
impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let jwt = jwt::extract_user_id_from_jwt(&req);
        match jwt {
            Ok(id) => { return ready(Ok(AuthUser { user_id: id })); }
            Err(e) => { return ready(Err(actix_web::error::ErrorUnauthorized("Unauthorised"))); }
        };
    }
}

// User ID as web param
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserUUID {
    pub u: String
}

// Auth
#[derive(serde::Serialize, serde::Deserialize, Debug, FromRow, Clone)]
pub struct PasswordChange {
    pub current_password: String,
    pub new_password: String
}

// Generate hashed password for test
#[derive(Deserialize)]
pub struct PasswordParams{
    pub user_password: String
}

// Cookie Claims
#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, FromRow, Clone)]
pub struct User {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub email: String,
    #[serde(rename = "password")]
    pub user_password: String,
    pub betacode: Option<String>
}
