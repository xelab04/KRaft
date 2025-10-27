#[macro_use]
extern crate actix_web;

use std::{env, io};

use actix_web::{middleware, web, App, HttpServer};

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

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");

    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "kraft.alexbissessur.dev".to_string());
    let config = AppConfig {
        environment: environment.clone(),
        host: host.clone(),
    };
    env_logger::init();

    // Will panic here if the db is unreachable :P
    let db_pool = db_connect::get_db_pool().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(middleware::Logger::default())
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
