// use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn get_db_pool() -> Result<PgPool, sqlx::Error> {
    sqlx::any::install_default_drivers();

    let user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let key = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let database = std::env::var("DB_DATABASE").unwrap_or_else(|_| "kraft".to_string());

    let db_url:&str = &format!("postgres://{}:{}@{}:{}/{}", user, key, host, port, database);

    PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
}
