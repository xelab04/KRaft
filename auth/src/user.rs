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
    useruuid_param: Option<web::Query<UserUUID>>
) -> HttpResponse {

    // If uuid is sent, then set that, otherwise default to jwt
    let jwt_user_id = &user.user_id;

    // check if is admin
    let is_admin: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = ? and admin = true)")
        .bind(jwt_user_id)
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(false);

    // If admin, let the user get the details for all users
    if is_admin {
        let req_uuid: String;
        match useruuid_param {
            Some(found_uuid) => req_uuid = found_uuid.u.clone(),
            None => req_uuid = user.user_id.clone()
        }

        let found_user: User = sqlx::query_as::<_, User>("SELECT user_id, username, email, uuid FROM users WHERE uuid = (?)")
            .bind(&req_uuid)
            .fetch_one(pool.as_ref())
            .await
            .unwrap();

        return HttpResponse::Ok().json(json!({"status": "success", "data": found_user}))
    }

    // get user details from database
    let user = sqlx::query_as::<_, User>("SELECT user_id, username, email, uuid FROM users WHERE user_id = (?)")
        .bind(&jwt_user_id)
        .fetch_one(pool.as_ref())
        .await;

    // return user if valid
    match user {
        Ok(user) => {
            HttpResponse::Ok().json(json!({"status": "success", "data": user}))
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({"status": "error", "message": "Internal Server Error"}));
        }
    }
}
