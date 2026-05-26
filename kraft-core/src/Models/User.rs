use actix_web::{Error, FromRequest, HttpRequest};
use futures_util::future::{Ready, ready};

use serde::{self, Deserialize, Serialize};
use sqlx::FromRow;
// use crate::{AppConfig, NtfyConfig, jwt};

use crate::Controllers::JWTController;

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct User {
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub email: String,
    #[serde(rename = "password")]
    pub user_password: String,
    pub betacode: Option<String>,
}

pub struct AuthUser {
    pub user_id: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserUUID {
    pub u: String,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        // if not running in prod, bypass auth
        // let config = req.app_data::<web::Data<AppConfig>>();
        // if let Some(cfg) = config {
        //     if cfg.environment != "PROD" {
        //         return ready(Ok(AuthUser { user_id: String::from("0") }))
        //     }
        // }

        let jwt = JWTController::extract_user_id_from_jwt(req);
        match jwt {
            Ok(id) => ready(Ok(AuthUser { user_id: id })),
            Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Unauthorised"))),
        }
    }
}
