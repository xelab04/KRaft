use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::prelude::FromRow;
use sqlx::MySqlPool;

use crate::jwt;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Cluster {
    name: String,
}

#[post("/api/create/clusters")]
pub async fn create_cluster(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    Json(cluster): Json<Cluster>,
) -> HttpResponse {
    let jwt = jwt::extract_user_id_from_jwt(&req);

    let user_id: String;
    match jwt {
        Ok(id) => {
            user_id = Some(id).unwrap();
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::Unauthorized().json("Unauthorized");
        }
    };

    let cluster_name = format!("{}-{}", user_id, cluster.name);



    // check for other clusters of the same name
    let count_same_name: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clusters WHERE name = ?")
        .bind(&cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();

    if count_same_name != 0 {
        return HttpResponse::BadRequest().json("Cluster with the same name already exists");
    }

    // call function to create cluster
    // theoretically just do k3k create <cluster_name>

    sqlx::query("INSERT INTO clusters (name, user_id) VALUES (?, ?)")
        .bind(&cluster_name)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        .unwrap();

    return HttpResponse::Ok().json("Cluster created successfully");
}

#[get("/api/get/clusters")]
pub async fn list(req: HttpRequest, pool: web::Data<MySqlPool>) -> HttpResponse {
    let jwt = jwt::extract_user_id_from_jwt(&req);

    let mut user_id = None;
    match jwt {
        Ok(id) => {
            user_id = Some(id);
        }
        Err(e) => {
            println!("Error: {:?}", e);
            // return HttpResponse::Unauthorized().json("Unauthorized")
        }
    };

    // use id to get from postgres

    let clusters = sqlx::query_as::<_, Cluster>("SELECT name FROM clusters WHERE user_id=(?)")
        .bind(user_id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    // let clusters = vec![
    //     Cluster {
    //         name: "Cluster 1".to_string(),
    //     },
    //     Cluster {
    //         name: "Cluster 2".to_string(),
    //     },
    // ];

    HttpResponse::Ok()
        .content_type("application/json")
        .json(clusters)
}
