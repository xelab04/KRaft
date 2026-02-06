use std::collections::BTreeMap;

use actix_web::web;
use actix_web::web::{Json, Path};
use actix_web::{HttpRequest, HttpResponse};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx;
use sqlx::prelude::FromRow;
use sqlx::MySqlPool;

use k3k_rs;
use kube::Client;
use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use random_word::Lang;
use tokio::fs;

use crate::{jwt, class};
use crate::validatename;
use crate::AppConfig;
use crate::tlssan;
use crate::ingress;
use crate::vcp;

use crate::class::{AuthUser, Cluster, ClusterCreateForm};

#[post("/api/create/clusters")]
pub async fn create(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    kubeclient: web::Data<Client>,
    config: web::Data<AppConfig>,
    user: AuthUser,
    Json(cluster): Json<ClusterCreateForm>,
) -> HttpResponse {
    // let jwt = jwt::extract_user_id_from_jwt(&req);

    // assume that for testing purposes the User's ID is 0
    // USER ID MANAGEMENT AND VALIDATION

    let user_id = user.user_id;
    let cluster_name = format!("k-{}-{}", user_id, cluster.name);
    let endpoint_string: String = format!("{}", cluster_name);
    let namespace = format!("k3k-{}", cluster_name);

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

    validated_tlssan_list.push(format!("{}.{}", endpoint_string, config.host));

    // validate cluster name
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
            serverLimit: Some(BTreeMap::from([
                ("cpu".to_string(), IntOrString::String("400m".to_string())),
                ("memory".to_string(), IntOrString::String("600Mi".to_string()))
            ])),
            workerLimit: Some(BTreeMap::from([
                ("cpu".to_string(), IntOrString::String("30m".to_string())),
                ("memory".to_string(), IntOrString::String("75Mi".to_string()))
            ])),
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

    //
    // CREATE NAMESPACE
    //

    if k3k_rs::namespace::exists(&kubeclient, &namespace).await.unwrap() {
        println!("Namespace already exists: {}", namespace);
    } else {
        println!("Namespace does not exist: {}", namespace);

        let namespace_schema = Namespace {
            metadata: ObjectMeta {
                name: Some(namespace.to_string()),
                labels: Some(BTreeMap::from([
                    ("policy.k3k.io/policy-name".to_string(), "kraft-vpc".to_string())
                ])),
                ..Default::default()
            },
            ..Default::default()
        };

        k3k_rs::namespace::create(&kubeclient, &namespace_schema).await.unwrap();
    }

    let response = k3k_rs::cluster::create(&kubeclient, &namespace, &cluster_schema).await;

    match response {
        Err(e) => {
            println!("{:?}", e);
            println!("Error creating cluster {}: {}", cluster_schema.metadata.name.unwrap(), e); return HttpResponse::BadGateway().json(e.to_string())
        }

        Ok(response) => {

            let title = "Cluster Created";
            let message = format!("Cluster {cluster_name} has just been created");

            if let Some(ntfy_config) = &config.ntfy {
                class::send_ntfy_notif(&ntfy_config.host, message.as_str(), title, &ntfy_config.basic_auth, &ntfy_config.token)
                    .await
                    .unwrap()
            }

            println!("Cluster created successfully");
        }
    }

    sqlx::query("INSERT INTO clusters (cluster_name, user_id, cluster_endpoint) VALUES (?, ?, ?)")
        .bind(&cluster_name)
        .bind(user_id)
        .bind(&endpoint_string)
        .execute(pool.get_ref())
        .await
        .unwrap();

    for (i, tlssan) in validated_tlssan_list.iter().enumerate() {
        ingress::traefik(&kubeclient, &cluster_name, &namespace, tlssan, i).await;
    }

    // vcp::create_default_vcp(&kubeclient, &cluster_name, &namespace).await;

    return HttpResponse::Ok().json("Cluster created successfully");
}

#[delete("/api/delete/cluster/{cluster_name}")]
pub async fn clusterdelete(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    cluster_name: web::Path<String>,
    kubeclient: web::Data<Client>,
    config: web::Data<AppConfig>,
    user: AuthUser
) -> HttpResponse{

    let raw_cluster_name = cluster_name.into_inner();
    let user_id = user.user_id;

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

    let namespace = format!("k3k-{}", raw_cluster_name);

    // let client = Client::try_default().await.unwrap();
    k3k_rs::cluster::delete(&kubeclient, namespace.as_str(), raw_cluster_name.as_str()).await.unwrap();

    k3k_rs::namespace::delete(&kubeclient, namespace.as_str()).await.unwrap();

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
    kubeclient: web::Data<Client>,
    config: web::Data<AppConfig>,
    user: AuthUser
) -> HttpResponse {

    let raw_cluster_name = cluster_name.into_inner();
    let user_id = user.user_id;

    // raw_cluster_name is 3-meow
    // so userid-clustername

    // check user_id and cluster_name in database
    let cluster_belongs_to_user: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = ? AND cluster_name = ?)")
        .bind(&user_id)
        .bind(&raw_cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to fetch cluster count");

    if !cluster_belongs_to_user {
        return HttpResponse::NotFound().json("Cluster not found");
    }

    // let client = Client::try_default().await.unwrap();
    let kconf;
    match k3k_rs::kubeconfig::get(&kubeclient, raw_cluster_name.as_str(), None).await {
        Ok(kubeconfig) => { kconf = kubeconfig }
        Err(e) => { return HttpResponse::Processing().json("Kubeconfig not found, wait a minute and try again.")}
    }

    let filename = "/kubeconfig.yaml";

    fs::write(&filename, kconf).await.unwrap();

    match std::fs::read_to_string(&filename) {
        Ok(file_contents) => {
            return HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}.yaml\"", raw_cluster_name)))
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
    user: AuthUser
) -> HttpResponse {
    let jwt = jwt::extract_user_id_from_jwt(&req);

    let user_id = user.user_id;
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
