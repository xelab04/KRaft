use actix_web::{web, HttpRequest, HttpResponse, delete};
use serde_json::json;
use sqlx::{MySqlPool, FromRow};
use serde_json;
use log::{info, error};
use anyhow::{anyhow, Result};
use std::sync::Arc; 
use k3k_rs::{
    client::Client as KubeClient, 
    cluster::delete as delete_cluster,
};

use crate::jwt; 

#[derive(serde::Serialize, serde::Deserialize, Debug, FromRow, Clone)]
pub struct User { // Made public for potential use outside the module
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub email: String,
    #[serde(rename = "password")]
    pub user_password: String,
    #[sqlx(skip)]
    pub betacode: Option<String>
}

// Struct to represent a cluster record from the database
#[derive(FromRow)]
struct ClusterRecord {
    namespace: String,
    cluster_name: String,
}

// --- Core Account Deletion Logic ---
async fn delete_account_cascade(
    pool: &MySqlPool,
    user_id: i32,
    kube_client: Arc<KubeClient>,
) -> Result<()> {
    
    // Start Transaction
    let mut tx = pool.begin().await?;

    // 1. Find all owned clusters from the database
    let clusters: Vec<ClusterRecord> = sqlx::query_as!(
        ClusterRecord,
        "SELECT namespace, cluster_name FROM clusters WHERE owner_id = ?",
        user_id
    )
    .fetch_all(&mut *tx) 
    .await?;

    // 2. Run k3k_rs::cluster::delete() to delete each cluster
    for cluster in clusters {
        info!(
            "Attempting Kubernetes cluster deletion: {}/{}",
            cluster.namespace, cluster.cluster_name
        );
        
        // Delete the cluster in Kubernetes
        delete_cluster(
            &kube_client, 
            &cluster.namespace, 
            &cluster.cluster_name
        ).await
         .map_err(|e| anyhow!("Failed to delete K8s cluster {}: {}", cluster.cluster_name, e))?;
         
         // The requirement suggests k3k-rs handles the DB entry deletion for the cluster.
         // If it doesn't, uncomment this:
         /*
         sqlx::query!(
             "DELETE FROM clusters WHERE owner_id = ? AND cluster_name = ?",
             user_id,
             cluster.cluster_name
         )
         .execute(&mut *tx)
         .await?;
         */
    }
    
    // 3. Clean up any remaining cluster entries owned by the user
    // This serves as a safety net in case k3k-rs::cluster::delete only deletes the K8s resource
    sqlx::query!(
        "DELETE FROM clusters WHERE owner_id = ?",
        user_id
    )
    .execute(&mut *tx)
    .await?;
    
    // 4. Delete the User's entry from the database
    let user_result = sqlx::query!(
        "DELETE FROM users WHERE user_id = ?",
        user_id
    )
    .execute(&mut *tx)
    .await?;

    if user_result.rows_affected() == 0 {
        // Rollback if the user wasn't found (shouldn't happen if JWT was valid)
        tx.rollback().await?;
        return Err(anyhow!("User ID {} not found or already deleted.", user_id));
    }

    // Commit Transaction
    tx.commit().await?;

    Ok(())
}

// --- NEW API Handler for Deletion ---
// Maps to: DELETE /auth/user/account
#[delete("/auth/user/account")]
pub async fn delete_account(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    kube_client: web::Data<Arc<KubeClient>>, 
) -> HttpResponse {

    // Obtain the User ID for the request (refer to fn validate_jwt)
    let jwt_result = jwt::extract_user_id_from_jwt(&req);

    let user_id = match jwt_result {
        Ok(id_str) => {
            // Need to convert String ID to i32
            match id_str.parse::<i32>() {
                Ok(id) => id,
                Err(_) => {
                    error!("JWT contained invalid user ID format: {}", id_str);
                    return HttpResponse::Unauthorized().json(json!({"status": "error", "message": "Invalid User ID format in token"}))
                },
            }
        }
        Err(e) => {
            info!("Unauthorized access attempt: {:?}", e);
            return HttpResponse::Unauthorized().json(json!({"status": "error", "message": "Unauthorized"}));
        }
    };
    
    info!("Attempting account deletion for user ID: {}", user_id);

    // Execute the core deletion logic
    match delete_account_cascade(
        pool.get_ref(), 
        user_id, 
        kube_client.get_ref().clone()
    ).await {
        Ok(_) => {
            HttpResponse::Ok().json(json!({"status": "success", "message": "Account and all associated resources successfully deleted."}))
        },
        Err(e) => {
            let error_message = format!("Account deletion failed for user {}: {}", user_id, e);
            error!("{}", error_message);
            // In case of a failure in K8s deletion, we return a server error
            HttpResponse::InternalServerError().json(json!({"status": "error", "message": error_message}))
        }
    }
}


// --- Existing API Handler (Kept as is) ---
#[actix_web::get("/auth/user/details")]
pub async fn details(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> HttpResponse {

    // get user id from request
    let jwt = jwt::extract_user_id_from_jwt(&req);

    let mut user_id: String = String::from("0");
    match jwt {
        Ok(id) => {
            user_id = Some(id).unwrap();
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::Unauthorized().json(json!({"status": "error", "message": "Unauthorized"}));
        }
    };

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
// Inside src/user.rs

// ... (all other necessary imports)

// The delete handler function MUST be public
#[delete("/auth/user/account")]
pub async fn delete_account( /* ... */ ) -> HttpResponse { 
    // ... implementation ...
}