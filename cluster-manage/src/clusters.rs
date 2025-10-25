use std::collections::BTreeMap;

use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx;
use sqlx::prelude::FromRow;
use sqlx::MySqlPool;

use k3k_rs;
use kube::Client;

use random_word::Lang;
use tokio::fs;

use crate::jwt;
use crate::validatename;
use crate::AppConfig;
use crate::tlssan;
use crate::ingress;


#[derive(Serialize, Deserialize, FromRow)]
pub struct Cluster {
    id: Option<i64>,
    name: String,
    endpoint: Option<String>
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct ClusterCreateForm {
    id: Option<i64>,
    name: String,
    tlssan_array: Option<Vec<String>>
}

#[post("/api/create/clusters")]
pub async fn create(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    Json(cluster): Json<ClusterCreateForm>,
) -> HttpResponse {
    let jwt = jwt::extract_user_id_from_jwt(&req);

    // assume that for testing purposes the User's ID is 0
    let mut user_id: String = String::from("0");
    match jwt {
        Ok(id) => {
            user_id = Some(id).unwrap();
        }
        Err(e) => {
            println!("Error: {:?}", e);
            if config.environment == "PROD" {
                return HttpResponse::Unauthorized().json(json!({"status": "error", "message": "Unauthorized"}));
            }
        }
    };

    let cluster_name = format!("{}-{}", user_id, cluster.name);

    // generate random string for whatever.kraft.alexb.dev
    let mut endpoint_string: String;
    loop {
        endpoint_string = format!("{}-{}", random_word::get(Lang::En), random_word::get(Lang::En));

        let count_with_same_endpoint: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clusters WHERE cluster_endpoint = (?)")
            .bind(&endpoint_string)
            .fetch_one(pool.get_ref())
            .await
            .unwrap();

        if count_with_same_endpoint == 0 {
            break
        }
    }

    // validate all TLS SANs
    let mut validated_tlssan_list = Vec::new();
    if let Some(value) = cluster.tlssan_array {
        let tlssan_list = Vec::from(value);
        for tlssan in tlssan_list.iter() {
            validated_tlssan_list.push(tlssan.trim().to_string());

            if !tlssan::validate_tlssan(tlssan.clone()).await.is_ok() {
                return HttpResponse::BadRequest().json("Invalid TLS-SAN format");
            }
        }
    }

    validated_tlssan_list.push(format!("{}.{} ", endpoint_string, config.host));


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

    let client = Client::try_default().await.unwrap();

    let namespace = format!("k3k-{}", cluster_name);

    let cluster_schema = k3k_rs::cluster::Cluster {
        metadata: kube::core::ObjectMeta {
            name: Some(cluster_name.clone()),
            namespace: Some(namespace.clone()),
            ..Default::default()
        },
        spec: k3k_rs::cluster::ClusterSpec {
            persistence: Some(k3k_rs::cluster::PersistenceSpec {
                r#type: Some("dynamic".to_string()),
                storageClassName: None,
                storageRequestSize: Some("2G".to_string()),
            }),
            tlsSANs: Some(validated_tlssan_list.clone()),
            expose: Some(k3k_rs::cluster::ExposeSpec {
                LoadBalancer: None,
                NodePort: None,
                Ingress: None
            }),
            sync: Some(k3k_rs::cluster::SyncSpec{
                ingresses: Some(k3k_rs::cluster::SyncResourceSpec {
                    enabled: true,
                    selector: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        status: None,
    };

    let response = k3k_rs::cluster::create(&client, &namespace, &cluster_schema).await;

    match response {
        Err(e) => {println!("Error creating cluster {}: {}", cluster_schema.metadata.name.unwrap(), e); return HttpResponse::BadGateway().json(e.to_string())}

        Ok(response) => {
            println!("Cluster created successfully");
        }
    }


    for (i, tlssan) in validated_tlssan_list.iter().enumerate() {
        ingress::traefik(&client, &cluster_name, &namespace, tlssan, i).await;
    }


    // Command::new("k3kcli")
    //     .arg("cluster")
    //     .arg("create")
    //     .arg(&server_arg_string)
    //     .arg(&cluster_name)
    //     .spawn()
    //     .expect("k3kcli command failed");

    sqlx::query("INSERT INTO clusters (cluster_name, user_id, cluster_endpoint) VALUES (?, ?, ?)")
        .bind(&cluster_name)
        .bind(user_id)
        .bind(&endpoint_string)
        .execute(pool.get_ref())
        .await
        .unwrap();

    return HttpResponse::Ok().json("Cluster created successfully");
}

#[delete("/api/delete/cluster/{cluster_name}")]
pub async fn clusterdelete(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    cluster_name: web::Path<String>,
    config: web::Data<AppConfig>,
) -> HttpResponse{

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
            if config.environment == "PROD" {
                return HttpResponse::Unauthorized().json("Unauthorized");
            }
        }
    };

    // check the user owns the cluster
    let cluster_count_belonging_to_user: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clusters WHERE user_id = ? AND cluster_name = ?")
        .bind(&user_id)
        .bind(&raw_cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to fetch cluster count");

    if cluster_count_belonging_to_user == 0 {
        return HttpResponse::NotFound().json("Cluster not found");
    }

    let namespace = format!("--namespace=k3k-{}", raw_cluster_name);

    let client = Client::try_default().await.unwrap();
    k3k_rs::cluster::delete(&client, namespace.as_str(), raw_cluster_name.as_str()).await.unwrap();

    let r = sqlx::query("DELETE FROM clusters WHERE user_id = ? AND cluster_name = ?")
        .bind(&user_id)
        .bind(&raw_cluster_name)
        .execute(pool.get_ref())
        .await;

    match r {
        Ok(_) => {
            HttpResponse::Ok().json("Success")
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(format!("Failed to delete cluster: {}", e))
        }
    }
}


#[get("/api/get/kubeconfig/{cluster_name}")]
pub async fn get_kubeconfig(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    cluster_name: web::Path<String>,
    config: web::Data<AppConfig>,
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
            if config.environment == "PROD" {
                return HttpResponse::Unauthorized().json("Unauthorized");
            }
        }
    };
    // raw_cluster_name is 3-meow
    // so userid-clustername

    // check user_id and cluster_name in database
    let cluster_count_belonging_to_user: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clusters WHERE user_id = ? AND cluster_name = ?")
        .bind(&user_id)
        .bind(&raw_cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to fetch cluster count");

    if cluster_count_belonging_to_user == 0 {
        return HttpResponse::NotFound().json("Cluster not found");
    }

    let client = Client::try_default().await.unwrap();
    let kconf = k3k_rs::kubeconfig::get(&client, raw_cluster_name.as_str(), None).await.unwrap();

    let filename = "/kubeconfig.yaml";

    fs::write(&filename, kconf).await.unwrap();

    match std::fs::read_to_string(&filename) {
        Ok(file_contents) => {
            return HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", raw_cluster_name)))
                .body(file_contents);
        },
        Err(e) => {
            println!("Error reading kubeconfig file: {}", e);
            return HttpResponse::InternalServerError().json("Failed to read kubeconfig file");
        }
    }
}

#[get("/api/get/clusters")]
pub async fn list(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
) -> HttpResponse {
    let jwt = jwt::extract_user_id_from_jwt(&req);

    let mut user_id:String = String::from("0");
    match jwt {
        Ok(id) => {
            user_id = id;
        }
        Err(e) => {
            println!("Error: {:?}", e);

            if config.environment == "PROD" {
                return HttpResponse::Unauthorized().json("Unauthorized")
            }
            // return HttpResponse::Unauthorized().json("Unauthorized")
        }
    };

    // use id to get from postgres

    let user_id_int: i32 = user_id.parse().unwrap_or(0);
    let clusters: Vec<Cluster> = sqlx::query_as::<_, Cluster>("SELECT cluster_id as id, cluster_name as name, cluster_endpoint as endpoint FROM clusters WHERE user_id=(?)")
        .bind(user_id_int)
        .fetch_all(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .json(clusters)
}
