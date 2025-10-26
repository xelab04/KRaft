#[macro_use]
extern crate actix_web;

use std::{env, io};
use log::{info, error};
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Arc;
// NEW: Import the KubeClient and anyhow::Result for error handling
use k3k_rs::client::Client as KubeClient; 
use anyhow::Result; 

mod auth;
mod db_connect;
mod jwt;
mod user; 

// Helper function to initialize the KubeClient
async fn initialize_kube_client() -> Result<Arc<KubeClient>> {
    info!("Initializing KubeClient from k3k-rs...");
    let client = KubeClient::new().await
        .map_err(|e| anyhow::anyhow!("Failed to initialize KubeClient: {}", e))?;
    
    Ok(Arc::new(client))
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    // --- Logging and Environment Setup ---
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "actix_web=debug,actix_server=info,auth_api=info");
    }
    env_logger::init();

    if env::var("JWT_SECRET").is_err() {
        unsafe { env::set_var("JWT_SECRET", "my_super_secure_jwt_secret_for_dev_only"); }
        info!("JWT_SECRET not found in .env, using default development key.");
    }

    // --- Data Initialization ---
    let db_pool = db_connect::get_db_pool().await
        .expect("FATAL: Failed to initialize Database Pool.");
    let db_pool_data = web::Data::new(db_pool);

    // NEW: Initialize and store the KubeClient
    let kube_client = initialize_kube_client().await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?; 
    let kube_client_data = web::Data::new(kube_client);

    // --- HTTP Server Setup ---
    info!("Starting server at 0.0.0.0:5000");

    HttpServer::new(move || {
        App::new()
            .app_data(db_pool_data.clone())      // Inject DB Pool
            .app_data(kube_client_data.clone())  // NEW: Inject Kube Client
            .wrap(middleware::Logger::default())
            
            // Existing Auth Services
            .service(auth::login)
            .service(auth::register)
            .service(auth::password)
            .service(auth::validate_jwt)
            .service(auth::changepwd)
            
            // User Services
            .service(user::details)
            .service(user::delete_account) // NEW: Register the delete service
    })
        .bind("0.0.0.0:5000")?
        .run()
        .await
}