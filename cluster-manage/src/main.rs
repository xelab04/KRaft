#[macro_use]
extern crate actix_web;

use std::{env, io};

use actix_web::{middleware, App, HttpServer};

// mod constants;
// mod like;
// mod response;
// mod tweet;
mod clusters;
mod jwt;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(clusters::list)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
