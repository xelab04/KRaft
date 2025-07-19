use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx;
use sqlx::prelude::FromRow;
use sqlx::MySqlPool;

use crate::jwt;

#[derive(Serialize, FromRow)]
pub struct Cluster {
    name: String,
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
