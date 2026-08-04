use actix_web::{HttpResponse, web};
use log::{error, info};
use serde_json::{self, json};
use sqlx::PgPool;

use crate::{
    Controllers::{
        DBHelper::{clusters, user, workspaces},
        JWTController,
    },
    Models::User::{AuthUser, User, UserUUID},
};

use k3k_rs;
use kube::Client;

#[get("/auth/user/list")]
pub async fn list(pool: web::Data<PgPool>, user: AuthUser) -> HttpResponse {
    let user_id: i32 = user.user_id.parse().unwrap();
    if !user::is_admin(&pool, &user_id).await.unwrap_or(false) {
        return HttpResponse::Forbidden().finish();
    }

    let users = user::list_users(&pool)
        .await
        .expect("failed to list all users in db");

    HttpResponse::Ok().json(users)
}

#[get("/auth/user/details")]
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

    // If admin, let the user get the details for any user
    if is_admin && let Some(found_uuid) = useruuid_param {
        info!("admin used, looking up {}", found_uuid.u);
        let found_user: User = sqlx::query_as::<_, User>(
            "SELECT user_id, username, email, uuid FROM users WHERE uuid = ($1)",
        )
        .bind(&found_uuid.u)
        .fetch_one(pool.as_ref())
        .await
        .unwrap();

        return HttpResponse::Ok().json(json!({"status": "success", "data": found_user}));
    }

    // get current user details from database
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

#[delete("/auth/user/delete")]
pub async fn user_delete(
    user: AuthUser,
    pool: web::Data<PgPool>,
    client: web::Data<Client>,
    _uuid_query_param: Option<web::Query<UserUUID>>,
) -> HttpResponse {
    let user_jwt = user.user_id;
    let int_user_id = user_jwt.parse::<i32>().unwrap();

    let clusters = clusters::list(&pool, &int_user_id).await.unwrap();

    // Delete all clusters associated with the user
    for cluster in clusters {
        let cluster_name = cluster.name;
        info!("Deleting cluster {}", cluster_name);
        let namespace = format!("k3k-{}", cluster_name);
        let r = k3k_rs::cluster::delete(&client, &namespace, &cluster_name).await;
        k3k_rs::namespace::delete(&client, namespace.as_str())
            .await
            .unwrap();
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
    info!("deleting user with user id: {} from database", int_user_id);
    workspaces::token_delete(&pool, &int_user_id).await.unwrap();
    workspaces::delete(&pool, &int_user_id).await.unwrap();
    user::delete(&pool, &int_user_id).await.unwrap();
    let delete_cookie = JWTController::del_cookie();

    HttpResponse::Ok()
        .cookie(delete_cookie)
        .json(json!({ "status": "success", "message": "success" }))
}

/// Validate the user account with a uuid sent to their mail
#[get("/auth/user/validate/{validation_code}")]
pub async fn validate(
    user: AuthUser,
    pool: web::Data<PgPool>,
    validation_code: web::Path<String>,
) -> HttpResponse {
    let raw_code = validation_code.into_inner();
    let int_user_id = user.user_id.parse::<i32>().unwrap();

    let user_verification_code = match user::get_verification_code(&pool, &int_user_id).await {
        Ok(code) => code,
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    if raw_code != user_verification_code {
        return HttpResponse::Unauthorized().finish();
    }

    user::validate(&pool, &user_verification_code)
        .await
        .unwrap();

    HttpResponse::Ok().json(json!({"status":"success", "message":"account verified, thank you"}))
}
