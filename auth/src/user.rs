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
    user_id: Option<i32>,
    username: Option<String>,
    email: String,
    #[serde(rename = "password")]
    user_password: String,
    #[sqlx(skip)]
    betacode: Option<String>
}


#[actix_web::get("/auth/user/details")]
pub async fn details(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    user: auth::AuthUser
) -> HttpResponse {


    let user_id = user.user_id;

    // get user id from request
    // let jwt = jwt::extract_user_id_from_jwt(&req);

    // let mut user_id: String = String::from("0");
    // match jwt {
    //     Ok(id) => {
    //         user_id = Some(id).unwrap();
    //     }
    //     Err(e) => {
    //         println!("Error: {:?}", e);
    //         return HttpResponse::Unauthorized().json(json!({"status": "error", "message": "Unauthorized"}));
    //     }
    // };



    // get user details from database
    let user = sqlx::query_as::<_, User>("SELECT user_id, username, email, password as user_password FROM users WHERE user_id = (?)")
        .bind(user_id)
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
