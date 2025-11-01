#[macro_use]
extern crate actix_web;

use std::{env, io};
use log::{info, error};

use actix_web::{middleware, web, App, HttpServer};

mod auth;
mod db_connect;
mod jwt;
mod user;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    unsafe { env::set_var("RUST_LOG", "actix_web=debug,actix_server=info"); }
    env_logger::init();

    if env::var("JWT_SECRET").is_err() {
        unsafe { env::set_var("JWT_SECRET", "my_super_secure_jwt_secret_for_dev_only"); }
        info!("JWT_SECRET not found in .env, using default development key.");
    }

    let db_pool = db_connect::get_db_pool().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(middleware::Logger::default())
            .service(auth::login)
            .service(auth::register)
            .service(auth::password)
            .service(auth::validate_jwt)
            .service(auth::get_user_id)
            .service(auth::changepwd)
            .service(user::details)
    })
        .bind("0.0.0.0:5000")?
        .run()
        .await
}
