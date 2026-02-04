#[macro_use]
extern crate actix_web;

use std::{env, io};
use log::{info, error};

// use actix_web::{middleware, web, App, HttpServer};

use actix_web::{
    HttpServer, Error, App,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{self, from_fn, Next},
    web
};
use kube::Client;

mod auth;
mod db_connect;
mod jwt;
mod user;
mod util;
mod class;


pub async fn update_cookie_middleware<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody + 'static,
{
    // pre-processing
    let user_id = jwt::extract_user_id_from_jwt(&req.request());

    if req.path() == "/auth/logout" {
        return next.call(req).await;
    }

    let mut response = next.call(req).await.unwrap();

    // post-processing
    match user_id {
        // if logged in (and thus holds token), refresh token
        Ok(uid) => {
            let jwt_token = jwt::create_jwt(uid);

            let cookie = jwt::create_cookie(&jwt_token);

            response.response_mut().add_cookie(&cookie).ok();
        }
        // if no uid, ex login/reg, don't add a cookie, right?
        Err(_) => {}
    };

    Ok(response)
}


#[actix_rt::main]
async fn main() -> io::Result<()> {
    unsafe { env::set_var("RUST_LOG", "actix_web=debug,actix_server=info"); }
    env_logger::init();

    if env::var("JWT_SECRET").is_err() {
        unsafe { env::set_var("JWT_SECRET", "my_super_secure_jwt_secret_for_dev_only"); }
        info!("JWT_SECRET not found in .env, using default development key.");
    }

    let db_pool = db_connect::get_db_pool().await.unwrap();
    let kube_client = Client::try_default().await.unwrap();
    let app_config = util::generate_appconfig();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(kube_client.clone()))
            .app_data(web::Data::new(app_config.clone()))
            .wrap(middleware::Logger::default())
            .wrap(from_fn(update_cookie_middleware))
            .service(auth::login)
            .service(auth::logout)
            .service(auth::register)
            .service(auth::password)
            .service(auth::validate_jwt)
            // .service(auth::get_user_id)
            .service(auth::changepwd)
            .service(user::details)
            .service(user::user_delete)
    })
        .bind("0.0.0.0:5000")?
        .run()
        .await
}
