#[macro_use]
extern crate actix_web;

use log::{error, info};
use rustls;
use std::{env, io, panic::PanicHookInfo};

use actix_web::{
    App, Error, HttpServer,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{self, Next, from_fn},
    web,
};

mod Controllers;
mod Models;

use Controllers::{ClusterController, UserController, WorkspaceController};
use kube::Client;

use crate::Controllers::{
    AuthController, BetacodeController, DBHelper, JWTController, LogsController,
    ResourceController, utils,
};
mod db_connect;

pub async fn update_cookie_middleware<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody + 'static,
{
    // pre-processing
    let user_id = JWTController::extract_user_id_from_jwt(req.request());
    let pool = req.app_data::<web::Data<sqlx::PgPool>>().cloned().unwrap();
    let app_config = req
        .app_data::<web::Data<crate::Models::Config::AppConfig>>()
        .cloned()
        .unwrap();

    let mut response = next.call(req).await.unwrap();

    // post-processing

    if let Ok(uid) = user_id {
        let jwt_token = JWTController::create_jwt(&pool, &app_config, &uid).await;
        let cookie = JWTController::create_cookie(jwt_token.as_str());
        response.response_mut().add_cookie(&cookie).ok();
    }
    Ok(response)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    unsafe {
        env::set_var("RUST_LOG", "actix_web=debug,actix_server=info,info");
    }
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("KRaft, created by Alex");
    info!("written in rust 🏳️‍⚧️");

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to set crypto provider to rustls");

    let panic_ntfy_config = utils::get_ntfy_config();
    let config = utils::generate_appconfig();

    let default_panic = std::panic::take_hook();
    fn get_message(info: &PanicHookInfo) -> String {
        let payload: &str = if let Some(s) = info.payload_as_str() {
            s
        } else {
            "Unknown panic?"
        };
        let location = if let Some(location) = info.location() {
            format!(" in file '{}' at line {}", location.file(), location.line())
        } else {
            "in an unknown location".to_string()
        };
        let message = format!("Panic: {} {}", payload, location);
        message
    }
    std::panic::set_hook(Box::new(move |info| {
        if let Some(ntfy) = &panic_ntfy_config {
            let message = get_message(info);

            utils::panic_ntfy(ntfy, &message, "Panic Occured");
        }
        default_panic(info)
    }));

    // Will panic here if the db is unreachable :P
    let db_pool = db_connect::get_db_pool().await.unwrap();
    let client = Client::try_default().await.unwrap();

    // check if a user already exists
    // and if a beta code already exists
    BetacodeController::first_startup(&db_pool).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(client.clone()))
            .wrap(middleware::Logger::default())
            .wrap(from_fn(update_cookie_middleware))
            .service(ClusterController::list)
            .service(ClusterController::create)
            .service(ClusterController::delete)
            .service(ClusterController::get_kubeconfig)
            .service(WorkspaceController::create)
            .service(WorkspaceController::create_token_for_terminal)
            .service(WorkspaceController::validate_terminal_access)
            .service(LogsController::getlogs)
            // .service(AuthController::generate_password)
            .service(AuthController::login)
            .service(AuthController::logout)
            .service(AuthController::register)
            .service(AuthController::validate_jwt)
            .service(AuthController::validate_admin)
            .service(AuthController::changepwd)
            .service(UserController::details)
            .service(UserController::user_delete)
            .service(UserController::list)
            .service(ResourceController::get_cluster_use)
            .service(ResourceController::get_namespace_use)
            .service(ClusterController::admin_list)
            .service(BetacodeController::create)
            .service(BetacodeController::update)
            .service(BetacodeController::new)
            .service(BetacodeController::delete)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
