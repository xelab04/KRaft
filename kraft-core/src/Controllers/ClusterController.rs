use k3k_rs::cluster::ExposeIngress;
use log::{error, info};
use std::collections::BTreeMap;

use actix_web::web::Json;
use actix_web::web::{self, Path};
use actix_web::{HttpRequest, HttpResponse};

use sqlx;
use sqlx::PgPool;

use k3k_rs;
use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Client;

// use crate::class;
// use crate::AppConfig;
use crate::Controllers::utils;
use crate::Controllers::{DBHelper::*, WorkspaceController};
use crate::Models::Config::AppConfig;

use crate::Models::Cluster::{Cluster, ClusterCreateForm, ClusterResourceConfig};
use crate::Models::User::AuthUser;

#[post("/api/create/clusters")]
pub async fn create(
    _req: HttpRequest,
    pool: web::Data<PgPool>,
    kubeclient: web::Data<Client>,
    config: web::Data<AppConfig>,
    user: AuthUser,
    Json(cluster): Json<ClusterCreateForm>,
) -> HttpResponse {
    let user_id = user.user_id;
    let cluster_name = format!("k-{}-{}", user_id, cluster.name);
    let endpoint_string: String = cluster_name.to_string();
    let namespace = format!("k3k-{}", cluster_name);

    // validate all TLS SANs
    let mut validated_tlssan_list = Vec::new();
    validated_tlssan_list.push(format!("{}.{}", endpoint_string, config.host));

    if let Some(value) = cluster.tlssan_array {
        let tlssan_list = value;
        for tlssan in tlssan_list.iter() {
            validated_tlssan_list.push(tlssan.trim().to_string());

            if utils::validate_tlssan(tlssan.clone()).await.is_err() {
                return HttpResponse::BadRequest().json("Invalid TLS-SAN format");
            }
        }
    }

    // validate cluster name
    if !utils::namevalid(&cluster_name) {
        return HttpResponse::BadRequest().json("Invalid Name");
    }

    // check for other clusters of the same name
    if clusters::same_name(&pool, &cluster_name).await.unwrap() {
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
                Ingress: Some(ExposeIngress {
                    ingressClassName: Some(config.network_config.ingress_class.clone()),
                    annotations: None,
                }),
            }),
            serverResources: Some(k3k_rs::cluster::ResourcesSpec {
                limits: Some(BTreeMap::from([
                    (
                        "cpu".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .servers
                            .limits
                            .cpu
                            .clone(),
                    ),
                    (
                        "memory".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .servers
                            .limits
                            .memory
                            .clone(),
                    ),
                ])),
                requests: Some(BTreeMap::from([
                    (
                        "cpu".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .servers
                            .requests
                            .cpu
                            .clone(),
                    ),
                    (
                        "memory".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .servers
                            .requests
                            .memory
                            .clone(),
                    ),
                ])),
            }),
            workerResources: Some(k3k_rs::cluster::ResourcesSpec {
                limits: Some(BTreeMap::from([
                    (
                        "cpu".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .workers
                            .limits
                            .cpu
                            .clone(),
                    ),
                    (
                        "memory".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .workers
                            .limits
                            .memory
                            .clone(),
                    ),
                ])),
                requests: Some(BTreeMap::from([
                    (
                        "cpu".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .workers
                            .requests
                            .cpu
                            .clone(),
                    ),
                    (
                        "memory".to_string(),
                        config
                            .resource_config
                            .cluster_resources
                            .workers
                            .requests
                            .memory
                            .clone(),
                    ),
                ])),
            }),
            sync: Some(k3k_rs::cluster::SyncSpec {
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

    // CREATE NAMESPACE
    if k3k_rs::namespace::exists(&kubeclient, &namespace)
        .await
        .unwrap()
    {
        info!("Namespace already exists: {}", namespace);
    } else {
        info!("Namespace does not exist: {}", namespace);

        let namespace_schema = Namespace {
            metadata: ObjectMeta {
                name: Some(namespace.to_string()),
                labels: Some(BTreeMap::from([(
                    "policy.k3k.io/policy-name".to_string(),
                    "kraft-vpc".to_string(),
                )])),
                ..Default::default()
            },
            ..Default::default()
        };

        k3k_rs::namespace::create(&kubeclient, &namespace_schema)
            .await
            .unwrap();
    }

    let response = k3k_rs::cluster::create(&kubeclient, &namespace, &cluster_schema).await;

    match response {
        Err(e) => {
            error!("{:?}", e);
            error!(
                "Error creating cluster {}: {}",
                cluster_schema.metadata.name.unwrap(),
                e
            );
            return HttpResponse::BadGateway().json(e.to_string());
        }

        Ok(_) => {
            let title = "Cluster Created";
            let message = format!("Cluster {cluster_name} has just been created");

            if let Some(ntfy_config) = &config.ntfy {
                utils::send_ntfy_notif(
                    &ntfy_config.host,
                    message.as_str(),
                    title,
                    &ntfy_config.basic_auth,
                    &ntfy_config.token,
                )
                .await
                .unwrap()
            }

            info!("Cluster created successfully");
        }
    }

    let int_user_id = user_id.parse::<i32>().unwrap();
    let cluster_id: i32 = sqlx::query_scalar(
        "INSERT INTO clusters (cluster_name, user_id, cluster_endpoint) VALUES ($1, $2, $3) RETURNING cluster_id",
    )
    .bind(&cluster_name)
    .bind(int_user_id)
    .bind(&endpoint_string)
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    for (_i, tlssan) in validated_tlssan_list.iter().enumerate() {
        let _ = k3k_rs::ingress::ingress_create(
            &kubeclient,
            &cluster_name,
            &namespace,
            tlssan,
            &config.network_config.ingress_class,
        )
        .await;
    }

    let ingress_path = format!("{}-wrk.{}", cluster_name, config.host);
    let workspace_name = format!("workspace-{}", cluster_name);

    info!("Creating a workspace for cluster: {}", cluster_name);
    WorkspaceController::core_workspace_create(
        &kubeclient,
        &pool,
        &config.host,
        ingress_path.as_str(),
        &config.network_config.ingress_class,
        &config.network_config.cluster_issuer,
        workspace_name.as_str(),
        cluster_name.as_str(),
        namespace.as_str(),
        &int_user_id,
        &cluster_id,
    )
    .await;

    // vcp::create_default_vcp(&kubeclient, &cluster_name, &namespace).await;

    HttpResponse::Ok().json("Cluster created successfully")
}

#[delete("/api/delete/cluster/{cluster_name}")]
pub async fn delete(
    pool: web::Data<PgPool>,
    cluster_name: web::Path<String>,
    kubeclient: web::Data<Client>,
    user: AuthUser,
) -> HttpResponse {
    let raw_cluster_name = cluster_name.into_inner();
    let user_id = user.user_id;
    let int_user_id = user_id.parse::<i32>().unwrap();

    // check cluster belongs to user
    if !clusters::name_belongs_to(&pool, &int_user_id, &raw_cluster_name)
        .await
        .unwrap()
    {
        return HttpResponse::NotFound().json("Cluster not found");
    }

    let namespace = format!("k3k-{}", raw_cluster_name);

    k3k_rs::cluster::delete(&kubeclient, namespace.as_str(), raw_cluster_name.as_str())
        .await
        .expect("cluster not found ");

    k3k_rs::namespace::delete(&kubeclient, namespace.as_str())
        .await
        .unwrap();

    let r = clusters::delete(&pool, &int_user_id, &raw_cluster_name).await;

    match r {
        Ok(_) => HttpResponse::Ok().json("Success"),
        Err(e) => HttpResponse::InternalServerError()
            .json(format!("Failed to delete cluster from db: {}", e)),
    }
}

#[get("/api/get/kubeconfig/{cluster_name}")]
pub async fn get_kubeconfig(
    pool: web::Data<PgPool>,
    cluster_name: web::Path<String>,
    kubeclient: web::Data<Client>,
    config: web::Data<AppConfig>,
    user: AuthUser,
) -> HttpResponse {
    let raw_cluster_name = cluster_name.into_inner();
    let user_id = user.user_id;
    let int_user_id = user_id.parse::<i32>().unwrap();

    // raw_cluster_name is 3-meow
    // so userid-clustername

    if !clusters::name_belongs_to(&pool, &int_user_id, &raw_cluster_name)
        .await
        .unwrap()
    {
        return HttpResponse::NotFound().json("Cluster not found");
    }

    // let client = Client::try_default().await.unwrap();

    let kconf = match k3k_rs::kubeconfig::get(&kubeclient, &raw_cluster_name, None).await {
        Ok(kubeconfig) => kubeconfig,
        Err(_) => {
            return HttpResponse::Processing()
                .json("Kubeconfig not found, wait a minute and try again.");
        }
    };

    // this feels so bad
    let mut new_kconf = String::new();
    for l in kconf.lines() {
        if l.starts_with("    server:") {
            new_kconf += format!(
                "    server: https://{}.{}:443 \n",
                raw_cluster_name, config.host
            )
            .as_str();
        } else {
            new_kconf += format!("{}\n", l).as_str();
        }
    }

    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}.yaml\"", raw_cluster_name),
        ))
        .body(new_kconf)
}

#[get("/api/get/clusters")]
pub async fn list(pool: web::Data<PgPool>, user: AuthUser) -> HttpResponse {
    let user_id = user.user_id;
    let user_id_int: i32 = user_id.parse().unwrap_or(0);

    let clusters: Vec<Cluster> = clusters::list(&pool, &user_id_int).await.unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .json(clusters)
}

#[get("/api/admin/get/clusters/{user_uuid}")]
pub async fn admin_list(
    pool: web::Data<PgPool>,
    user: AuthUser,
    user_uuid: Path<String>,
) -> HttpResponse {
    let user_id = user.user_id;
    let user_id_int: i32 = user_id.parse().unwrap_or(0);
    let target_uuid: String = user_uuid.into_inner();

    if !user::is_admin(&pool, &user_id_int).await.unwrap() {
        return HttpResponse::Unauthorized().finish();
    }

    let target_user_id = user::get_id_from_uuid(&pool, &target_uuid).await.unwrap();

    let clusters: Vec<Cluster> = clusters::list(&pool, &target_user_id).await.unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .json(clusters)
}
