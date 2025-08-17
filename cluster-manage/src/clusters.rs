use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::prelude::FromRow;
use sqlx::MySqlPool;

use std::process::Command;


use crate::jwt;
use crate::validatename;

use random_word::Lang;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Cluster {
    name: String,
}

#[post("/api/create/clusters")]
pub async fn create(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    Json(cluster): Json<Cluster>,
) -> HttpResponse {
    let jwt = jwt::extract_user_id_from_jwt(&req);

    let mut user_id: String = String::from("0");
    match jwt {
        Ok(id) => {
            user_id = Some(id).unwrap();
        }
        Err(e) => {
            println!("Error: {:?}", e);
            // #[PROD]
            // return HttpResponse::Unauthorized().json("Unauthorized");
        }
    };

    let cluster_name = format!("{}-{}", user_id, cluster.name);

    if !validatename::namevalid(&cluster_name) {
        return HttpResponse::BadRequest().json("Invalid Name");
    }

    // check for other clusters of the same name
    let count_same_name: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clusters WHERE cluster_name = ?")
        .bind(&cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();

    if count_same_name != 0 {
        return HttpResponse::BadRequest().json("Cluster with the same name already exists");
    }

    let mut endpoint_string = String::new();
    loop {
        endpoint_string = format!("{}{}", random_word::get(Lang::En), random_word::get(Lang::En));

        let count_with_same_endpoint: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clusters WHERE cluster_endpoint = (?)")
            .bind(&endpoint_string)
            .fetch_one(pool.get_ref())
            .await
            .unwrap();

        if count_with_same_endpoint == 0 {
            break
        }
    }

    // #[PROD]
    // call function to create cluster
    // theoretically just do k3k create <cluster_name>

    println!("{}", cluster_name);

    Command::new("k3kcli")
        .arg("cluster")
        .arg("create")
        .arg(&cluster_name)
        .spawn()
        .expect("k3kcli command failed");

    sqlx::query("INSERT INTO clusters (cluster_name, user_id, cluster_endpoint) VALUES (?, ?, ?)")
        .bind(&cluster_name)
        .bind(user_id)
        .bind(&endpoint_string)
        .execute(pool.get_ref())
        .await
        .unwrap();

    return HttpResponse::Ok().json("Cluster created successfully");
}

#[get("/api/get/kubeconfig/{cluster_name}")]
pub async fn get_kubeconfig(
    req: HttpRequest,
    cluster_name: web::Path<String>,
    pool: web::Data<MySqlPool>,
) -> HttpResponse {

    let raw_cluster_name = cluster_name.into_inner();

    let jwt = jwt::extract_user_id_from_jwt(&req);

    let mut user_id: String = String::from("0");
    match jwt {
        Ok(id) => {
            user_id = Some(id).unwrap();
        }
        Err(e) => {
            println!("Error: {:?}", e);
            // #[PROD]
            // return HttpResponse::Unauthorized().json("Unauthorized");
        }
    };

    let cluster_name = format!("{}-{}", user_id, raw_cluster_name);

    Command::new("k3kcli")
        .arg("kubeconfig")
        .arg("generate")
        .arg(format!("--namespace=k3k-{}", cluster_name))
        .arg(format!("--name={}", cluster_name))
        .output()
        .expect("k3kcli command failed");

    return HttpResponse::Ok().json("Kubeconfig generated successfully");
}

#[get("/api/get/clusters")]
pub async fn list(
    req: HttpRequest,
    pool: web::Data<MySqlPool>
) -> HttpResponse {
    let jwt = jwt::extract_user_id_from_jwt(&req);

    let mut user_id:String = String::from("0");
    match jwt {
        Ok(id) => {
            user_id = id;
        }
        Err(e) => {
            println!("Error: {:?}", e);
            // #[PROD]
            // return HttpResponse::Unauthorized().json("Unauthorized")
        }
    };

    // use id to get from postgres

    let clusters: Vec<String> = sqlx::query_scalar("SELECT cluster_name FROM clusters WHERE user_id=(?)")
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


#[delete("/api/delete/{cluster_name}")]
pub async fn delete(
    cluster_name: web::Path<String>,
    req: HttpRequest,
    pool: web::Data<MySqlPool>
) -> HttpResponse {
    // let cluster_name = query.cluster_id;
    let cluster_name = cluster_name.into_inner();

    let jwt = jwt::extract_user_id_from_jwt(&req);

    let user_id: String; // = String::from("-1");
    match jwt {
        Ok(id) => {
            user_id = id //Some(id);
        }
        Err(e) => {
            println!("Error: {:?}", e);
            // #[PROD]
            user_id = String::from("1");
            // return HttpResponse::Unauthorized().json("Unauthorized")
        }
    };

    // use id to get from postgres
    // add onto this - return 404 if cluster doesn't exist
    let cluster_owner: String = sqlx::query_scalar("SELECT user_id FROM clusters WHERE cluster_name = ?")
        .bind(&cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();

    if cluster_owner != user_id {
        return HttpResponse::Forbidden().json("This cluster is not yours.")
    }

    // call function to delete cluster
    // theoretically just do k3k delete <cluster_name>

    sqlx::query("DELETE FROM clusters WHERE cluster_name = ?")
        .bind(&cluster_name)
        .execute(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Ok().json("Cluster deleted successfully")
}
