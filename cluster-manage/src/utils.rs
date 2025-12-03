use actix_web::{FromRequest, Error, HttpRequest};
use futures_util::future::{ready, Ready};

use crate::jwt;


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
