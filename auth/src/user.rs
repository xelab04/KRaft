use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::FromRow;
use serde_json;
use log::{info};

use crate::jwt;
use crate::auth;


#[derive(serde::Serialize, serde::Deserialize, Debug, FromRow, Clone)]
struct User {
    user_id: i32,
    username: String,
    email: String,
    uuid: String,
    #[serde(rename = "password")]
    #[sqlx(skip)]
    user_password: String,
    #[sqlx(skip)]
    betacode: Option<String>
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserUUID {
    u: String
}

#[actix_web::get("/auth/user/details")]
pub async fn details(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    user: auth::AuthUser,
    user_uuid: web::Query<UserUUID>
) -> HttpResponse {

    let user_id = user.user_id;
    let user_uuid = &user_uuid.u;

    // get user details from database
    let user = sqlx::query_as::<_, User>("SELECT user_id, username, email, uuid FROM users WHERE uuid = (?)")
        .bind(user_uuid)
        .fetch_one(pool.as_ref())
        .await;

    // return user if valid
    match user {
        Ok(user) => {
            // Check that the user who made the request matches the uuid fetched
            if user_id != user.user_id.to_string() {
                return HttpResponse::Unauthorized().json(json!({"status": "error", "message": "Unauthorized"}));
            }

            HttpResponse::Ok().json(json!({"status": "success", "data": user}))
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({"status": "error", "message": "Internal Server Error"}));
        }
    }
}
