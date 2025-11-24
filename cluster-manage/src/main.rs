#[macro_use]
extern crate actix_web;

use std::{env, io};

use actix_web::{
    HttpServer, Error, App,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{self, from_fn, Next},
    web
};
use actix_web::{cookie::Cookie, cookie::SameSite};
use actix_web::cookie::time::Duration;

use kube::Client;

mod clusters;
mod db_connect;
mod validatename;
mod jwt;
mod tlssan;
mod ingress;
mod logs;

#[derive(Clone)]
pub struct AppConfig {
    pub environment: String,
    pub host: String,
}

pub async fn update_cookie_middleware<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody + 'static,
{
    // pre-processing
    let user_id = jwt::extract_user_id_from_jwt(&req.request());

    let mut response = next.call(req).await.unwrap();

    // post-processing
    match user_id {
        Ok(uid) => {
            let jwt_token = jwt::create_jwt(uid);

            let cookie = jwt::create_cookie(&jwt_token);

            response.response_mut().add_cookie(&cookie).ok();
        }
        Err(_) => {}
    };

    Ok(response)
}


#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");

    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "kraftcloud.dev".to_string());
    let config = AppConfig {
        environment: environment.clone(),
        host: host.clone(),
    };
    env_logger::init();

    // Will panic here if the db is unreachable :P
    let db_pool = db_connect::get_db_pool().await.unwrap();
    let client = Client::try_default().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(client.clone()))
            .wrap(middleware::Logger::default())
            .wrap(from_fn(update_cookie_middleware))
            .service(clusters::list)
            .service(clusters::create)
            .service(clusters::clusterdelete)
            .service(clusters::get_kubeconfig)
            .service(logs::getlogs)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
