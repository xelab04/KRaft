use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::FromRow;
use serde_json;
use log::{info};

use crate::jwt;
use crate::util;
use crate::class::{AuthUser, UserUUID, User};

use k3k_rs;
use kube::Client;

// #[derive(serde::Serialize, serde::Deserialize, Debug, FromRow, Clone)]
// struct User {
//     user_id: i32,
//     username: String,
//     email: String,
//     uuid: String,
//     #[serde(rename = "password")]
//     #[sqlx(skip)]
//     user_password: String,
//     #[sqlx(skip)]
//     betacode: Option<String>
// }



#[actix_web::get("/auth/user/details")]
pub async fn details(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    user: AuthUser,
    useruuid_param: Option<web::Query<UserUUID>>
) -> HttpResponse {

    // If uuid is sent, then set that, otherwise default to jwt
    let jwt_user_id = &user.user_id;

    // check if is admin
    let is_admin: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1 and admin = true)")
        .bind(jwt_user_id)
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(false);

    // If admin, let the user get the details for all users
    if is_admin {
        let req_uuid: String;
        match useruuid_param {
            Some(found_uuid) => { println!("admin used, userid specified"); req_uuid = found_uuid.u.clone(); }
            None => {
                println!("admin used, no userid specified");
                req_uuid = user.user_id.clone();
                let found_user: User = sqlx::query_as::<_, User>("SELECT user_id, username, email, uuid FROM users WHERE user_id = ($1)")
                    .bind(&req_uuid)
                    .fetch_one(pool.as_ref())
                    .await
                    .unwrap();
                return HttpResponse::Ok().json(json!({"status": "success", "data": found_user}))
            }
        }

        let found_user: User = sqlx::query_as::<_, User>("SELECT user_id, username, email, uuid FROM users WHERE uuid = ($1)")
            .bind(&req_uuid)
            .fetch_one(pool.as_ref())
            .await
            .unwrap();

        return HttpResponse::Ok().json(json!({"status": "success", "data": found_user}))
    }

    // get user details from database
    let user = sqlx::query_as::<_, User>("SELECT user_id, username, email, uuid, password as user_password, betacode FROM users WHERE user_id = ($1)")
        .bind(&jwt_user_id)
        .fetch_one(pool.as_ref())
        .await;

    // return user if valid
    match user {
        Ok(mut user) => {
            user.user_password = String::new();
            user.betacode = None;

            HttpResponse::Ok().json(json!({"status": "success", "data": user}))
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({"status": "error", "message": "Internal Server Error"}));
        }
    }
}


#[actix_web::delete("/auth/user/delete")]
pub async fn user_delete (
    user: AuthUser,
    pool: web::Data<MySqlPool>,
    client: web::Data<Client>,
    uuid_query_param: Option<web::Query<UserUUID>>,
) -> HttpResponse {

    let user_jwt = user.user_id;

    // let client = Client::try_default().await.unwrap();

    let is_admin: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1 and admin = true)")
        .bind(&user_jwt)
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(false);


    let cluster_names: Vec<String> = sqlx::query_scalar("SELECT cluster_name FROM clusters WHERE user_id = $1")
        .bind(&user_jwt)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_default();

    // Delete all clusters associated with the user
    for cluster_name in cluster_names {
        println!("Deleting cluster {}", cluster_name);
        let namespace = format!("k3k-{}", cluster_name);
        let r = k3k_rs::cluster::delete(&client, &namespace.as_str(), &cluster_name.as_str()).await;
        match r {
            Ok(_) => {
                println!("Cluster {} deleted successfully", cluster_name);
                sqlx::query("DELETE FROM clusters WHERE user_id = $1 AND cluster_name = $2")
                    .bind(&user_jwt)
                    .bind(&cluster_name)
                    .execute(pool.as_ref())
                    .await
                    .unwrap_or_default();
            }
            Err(e) => {
                println!("Error deleting cluster {}: {:?}", cluster_name, e);
                return HttpResponse::InternalServerError().json(json!({"status": "error", "message": format!("Failed deleting cluster: {}", cluster_name)}));
            }
        }
    }

    // Delete user from database
    sqlx::query("DELETE FROM users WHERE user_id = $1")
        .bind(&user_jwt)
        .execute(pool.as_ref())
        .await
        .unwrap_or_default();

    let delete_cookie = jwt::del_cookie();

    // return HttpResponse::Ok().finish();
    return HttpResponse::Ok()
        .cookie(delete_cookie)
        .json(json!({ "status": "success", "message": "success" }));
}


#[actix_web::get("/auth/validate/{token}")]
pub async fn validate (
    user: AuthUser,
    pool: web::Data<MySqlPool>,
    token: web::Path<String>,
) -> HttpResponse {

    let raw_token = token.into_inner();

    let possible_stored_user_token: Result<String, sqlx::Error> = sqlx::query_scalar("SELECT verification_code FROM users WHERE user_id = ($1)")
        .bind(&user.user_id)
        .fetch_one(pool.as_ref())
        .await;

    let db_user_token;
    match possible_stored_user_token {
        Ok(stored_user_token) => { db_user_token = stored_user_token; }
        Err(_) => { return HttpResponse::Unauthorized().finish() }
    }

    if !util::check_passwords_match(&raw_token, &db_user_token) {
        return HttpResponse::Unauthorized().finish();
    }

    let _r = sqlx::query("UPDATE users SET verified_email = true WHERE verification_code = ($1)")
        .bind(&db_user_token)
        .execute(pool.as_ref())
        .await
        .unwrap();

    return HttpResponse::Ok().json(json!({"status":"success", "message":"account validated, thank you"}));
}
