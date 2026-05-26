use actix_web::{HttpResponse, web};
use log::{error, info};
use serde_json;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    Controllers::{
        DBHelper::{clusters, user},
        JWTController, utils,
    },
    Models::User::{AuthUser, User, UserUUID},
};

use k3k_rs;
use kube::Client;

#[actix_web::get("/auth/user/details")]
pub async fn details(
    pool: web::Data<PgPool>,
    user: AuthUser,
    useruuid_param: Option<web::Query<UserUUID>>,
) -> HttpResponse {
    // If uuid is sent, then set that, otherwise default to jwt
    let jwt_user_id = &user.user_id;
    let int_user_id = jwt_user_id.parse::<i32>().unwrap();

    // check if is admin
    let is_admin = user::is_admin(&pool, &int_user_id).await.unwrap_or(false);

    // If admin, let the user get the details for all users
    if is_admin {
        let req_uuid: String = match useruuid_param {
            Some(found_uuid) => {
                info!("admin used, no userid specified");
                found_uuid.u.clone()
            }
            None => {
                println!("admin used, no userid specified");

                let int_user_id = user.user_id.clone().parse::<i32>().unwrap();
                let found_user = user::get_details(&pool, &int_user_id).await.unwrap();
                return HttpResponse::Ok().json(json!({"status": "success", "data": found_user}));
            }
        };

        let found_user: User = sqlx::query_as::<_, User>(
            "SELECT user_id, username, email, uuid FROM users WHERE uuid = ($1)",
        )
        .bind(&req_uuid)
        .fetch_one(pool.as_ref())
        .await
        .unwrap();

        return HttpResponse::Ok().json(json!({"status": "success", "data": found_user}));
    }

    // get user details from database
    match user::get_details(&pool, &int_user_id).await {
        Err(e) => {
            error!("Error: {:?}", e);
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Internal Server Error"}))
        }
        Ok(mut user) => {
            user.user_password = String::new();
            user.betacode = None;

            HttpResponse::Ok().json(json!({"status": "success", "data": user}))
        }
    }
}

#[actix_web::delete("/auth/user/delete")]
pub async fn user_delete(
    user: AuthUser,
    pool: web::Data<PgPool>,
    client: web::Data<Client>,
    _uuid_query_param: Option<web::Query<UserUUID>>,
) -> HttpResponse {
    let user_jwt = user.user_id;
    let int_user_id = user_jwt.parse::<i32>().unwrap();

    let _is_admin = user::is_admin(&pool, &int_user_id).await.unwrap_or(false);

    let clusters = clusters::list(&pool, &int_user_id).await.unwrap();

    // Delete all clusters associated with the user
    for cluster in clusters {
        let cluster_name = cluster.name;
        info!("Deleting cluster {}", cluster_name);
        let namespace = format!("k3k-{}", cluster_name);
        let r = k3k_rs::cluster::delete(&client, &namespace, &cluster_name).await;
        match r {
            Ok(_) => {
                info!("Cluster {} deleted successfully", cluster_name);
                clusters::delete(&pool, &int_user_id, &cluster_name)
                    .await
                    .unwrap_or_default();
            }
            Err(e) => {
                error!("Error deleting cluster {}: {:?}", cluster_name, e);
                return HttpResponse::InternalServerError().json(json!({"status": "error", "message": format!("Failed deleting cluster: {}", cluster_name)}));
            }
        }
    }

    // Delete user from database
    user::delete(&pool, &int_user_id).await.unwrap_or_default();
    let delete_cookie = JWTController::del_cookie();

    // return HttpResponse::Ok().finish();
    HttpResponse::Ok()
        .cookie(delete_cookie)
        .json(json!({ "status": "success", "message": "success" }))
}

/// Validate the user account with a token sent to their mail
#[actix_web::get("/auth/validate/{token}")]
pub async fn validate(
    user: AuthUser,
    pool: web::Data<PgPool>,
    token: web::Path<String>,
) -> HttpResponse {
    let raw_token = token.into_inner();
    let int_user_token = user.user_id.parse::<i32>().unwrap();

    let user_token = match user::get_validation_token(&pool, &int_user_token).await {
        Ok(token) => token,
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    if !utils::check_passwords_match(&raw_token, &user_token) {
        return HttpResponse::Unauthorized().finish();
    }

    user::validate(&pool, &user_token).await.unwrap();

    HttpResponse::Ok().json(json!({"status":"success", "message":"account validated, thank you"}))
}
