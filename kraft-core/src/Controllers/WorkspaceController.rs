use actix_web::HttpResponse;
use actix_web::web;
use actix_web::web::Json;
use actix_web::web::to;
use k3k_rs::cluster;
use kube::core::ErrorResponse;
use serde::{self, Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;

use chrono;
use log::info;
use uuid::Uuid;

use sqlx;
use sqlx::PgPool;

use crate::Controllers::DBHelper::{clusters, workspaces};
use crate::Models::Config::AppConfig;
use crate::Models::User::AuthUser;
use crate::utils;

use kube::{
    Client,
    api::{Api, PostParams},
    core::{ApiResource, DynamicObject, GroupVersionKind},
};
use serde_json::json;

pub async fn ingress(
    client: &Client,
    cluster_name: &str,
    namespace: &str,
    ingress_path: &str,
    ingress_class: &str,
) {
    let gvk = GroupVersionKind::gvk("networking.k8s.io", "v1", "Ingress");
    let ar = ApiResource::from_gvk(&gvk);

    let ingress_handler: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);
    let ingress_name = format!("workspace-{}", cluster_name);

    match ingress_handler.get(&ingress_name).await {
        Ok(_) => {
            info!("ingress already exists, skipping");
            return;
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            info!("ingress does not exist, proceeding with creation");
        }
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }

    let ingress = json!({
        "apiVersion": "networking.k8s.io/v1",
        "kind": "Ingress",
        "metadata": {
            "name": ingress_name,
            "namespace": namespace,
            "annotations": {
                "cert-manager.io/cluster-issuer": "prod-issuer"
            }
        },
        "spec": {
            "ingressClassName": ingress_class,
            "tls": [{
                "hosts": [ingress_path],
                "secretName": format!("{}-tls", cluster_name)
            }],
            "rules": [{
                "host": ingress_path,
                "http": {
                    "paths": [{
                        "path": "/",
                        "pathType": "Prefix",
                        "backend": {
                            "service": {
                                "name": format!("workspace-{}",cluster_name),
                                "port": {"number": 8080}
                            }
                        }
                    }]
                }
            }]
        }
    });

    let pp = PostParams::default();
    let ingressroute: DynamicObject = serde_json::from_value(ingress).unwrap();

    let _created = ingress_handler.create(&pp, &ingressroute).await.unwrap();
}

pub async fn service(client: &Client, cluster_name: &str, namespace: &str) {
    // define CRD type
    let gvk = GroupVersionKind::gvk("", "v1", "Service");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let service_handler: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);
    let service_name = format!("workspace-{}", cluster_name);

    match service_handler.get(&service_name).await {
        Ok(_) => {
            info!("service already exists, skipping");
            return;
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            info!("service does not exist, proceeding");
        }
        Err(e) => {
            panic!("error checking service existence: {}", e);
        }
    }

    // json cause im lazy
    let svc = json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": service_name,
            "namespace": namespace
        },
        "spec": {
            "selector": {
                "app": "workspace"
            },
            "ports": [{
                "protocol": "TCP",
                "port": 8080,
                "targetPort": 8080
            }],
            "type": "ClusterIP"
        }
    });

    let pp = PostParams::default();
    let ingressroute: DynamicObject = serde_json::from_value(svc).unwrap();

    let _created = service_handler.create(&pp, &ingressroute).await.unwrap();
}

pub async fn statefulset(
    client: &Client,
    cluster_name: &str,
    namespace: &str,
    cluster_id: &i32,
    host: &str,
) {
    // define CRD type
    let gvk = GroupVersionKind::gvk("apps", "v1", "StatefulSet");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let statefulset_handler: Api<DynamicObject> =
        Api::namespaced_with(client.clone(), namespace, &ar);
    let statefulset_name = format!("workspace");

    match statefulset_handler.get(&statefulset_name).await {
        Ok(_) => {
            info!("statefulset already exists, skipping");
            return;
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            info!("statefulset does not exist, proceeding");
        }
        Err(e) => {
            panic!("error checking statefulset existence: {}", e);
        }
    }

    // json cause im lazy
    let ss = json!({
        "apiVersion": "apps/v1",
        "kind": "StatefulSet",
        "metadata": {
            "name": statefulset_name,
            "namespace": namespace
        },
        "spec": {
            "selector": {
                "matchLabels": {
                    "app": "workspace"
                }
            },
            "serviceName": format!("workspace-{}", cluster_name),
            "replicas": 1,
            "minReadySeconds": 10,
            "template": {
                "metadata": {"labels":{"app": "workspace"}},
                "spec": {
                    "terminationGracePeriodSeconds": 10,
                    "imagePullSecrets": [{"name": "regcred"}],
                    "automountServiceAccountToken": false,
                    "securityContext": {
                      "fsGroup": 1000,
                      "runAsNonRoot": true
                    },
                    "containers": [
                        {
                            "name": "workspace-proxy",
                            "image": "ghcr.io/xelab04/kraft-workspace-proxy:latest",
                            "imagePullPolicy": "Always",
                            "env": [
                                {
                                    "name": "HOST",
                                    "value": String::from(host)
                                },
                                {
                                    "name": "CLUSTER_ID",
                                    "value": format!("{}", cluster_id)
                                }
                            ],
                            "ports": [{
                                "containerPort": 8080,
                                "name": "web"
                            }],
                            "resources": {
                                "limits": {
                                    "cpu": "50m",
                                    "memory": "100M"
                                }
                            },
                            "securityContext": {
                                "allowPrivilegeEscalation": false,
                                "runAsUser": 1000
                            }
                        },
                        {
                            "name": "workspace",
                            "image": "ghcr.io/xelab04/kraft-workspace:latest",
                            "imagePullPolicy": "Always",
                            "ports": [{
                                "containerPort": 7681,
                                "name": "ttyd"
                            }],
                            "volumeMounts": [{
                                "name": "kubeconfig",
                                "mountPath": "/home/kraft/.kube/config",
                                "subPath": "config"
                            }],
                            "resources": {
                                "limits": {
                                    "cpu": "100m",
                                    "memory": "50M"
                                }
                            },
                            "securityContext": {
                                "allowPrivilegeEscalation": false,
                                "runAsUser": 1000
                            }
                        }
                    ],
                    "volumes": [{
                        "name": "kubeconfig",
                        "secret": {
                            "secretName": format!("k3k-{}-kubeconfig", cluster_name),
                            "defaultMode": 0444,
                            "items": [{
                                "key": "kubeconfig.yaml",
                                "path": "config"
                            }]
                        }
                    }]
                }

            }
        }
    });

    let pp = PostParams::default();
    let statefulset: DynamicObject = serde_json::from_value(ss).unwrap();

    let _created = statefulset_handler.create(&pp, &statefulset).await.unwrap();
}

pub async fn netpol(client: &Client, cluster_name: &str, namespace: &str) {
    // define CRD type
    let gvk = GroupVersionKind::gvk("networking.k8s.io", "v1", "NetworkPolicy");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let netpol_handler: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);
    let netpol_name = format!("workspace-{}", cluster_name);

    match netpol_handler.get(&netpol_name).await {
        Ok(_) => {
            info!("netpol already exists, skipping");
            return;
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            info!("netpol does not exist, proceeding");
        }
        Err(e) => {
            panic!("error checking netpol existence: {}", e);
        }
    }

    let netpol = json!({
        "apiVersion": "networking.k8s.io/v1",
        "kind": "NetworkPolicy",
        "metadata": {
            "name": netpol_name,
            "namespace": namespace
        },
        "spec": {
            "policyTypes": ["Egress"],
            "podSelector": {
                "matchLabels": {
                    "app": "workspace"
                }
            },
            "egress": [{
                "to": [{
                    "namespaceSelector": {
                        "matchLabels": {
                            "kubernetes.io/metadata.name": "kraft"
                        }
                    }
                }]
            }]
        }
    });

    let pp = PostParams::default();
    let netpolresource: DynamicObject = serde_json::from_value(netpol).unwrap();

    let _created = netpol_handler.create(&pp, &netpolresource).await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct WorkspaceCreate {
    pub name: String,
}

pub async fn core_workspace_create(
    kubeclient: &Client,
    pool: &web::Data<Pool<Postgres>>,
    host: &str,
    ingress_path: &str,
    ingress_class: &str,
    workspace_name: &str,
    cluster_name: &str,
    namespace: &str,
    user_id: &i32,
    cluster_id: &i32,
) {
    netpol(&kubeclient, cluster_name, namespace).await;

    statefulset(kubeclient, cluster_name, namespace, cluster_id, host).await;

    service(&kubeclient, cluster_name, namespace).await;

    ingress(
        kubeclient,
        cluster_name,
        namespace,
        ingress_path,
        ingress_class,
    )
    .await;

    workspaces::create(pool, workspace_name, &cluster_name, user_id)
        .await
        .expect("failed adding workspace to db");
}

#[post("/api/create/workspaces")]
pub async fn create(
    pool: web::Data<PgPool>,
    kubeclient: web::Data<Client>,
    config: web::Data<AppConfig>,
    user: AuthUser,
    Json(workspace): Json<WorkspaceCreate>,
) -> HttpResponse {
    let user_id = user.user_id;
    let cluster_name = workspace.name;
    let namespace = format!("k3k-{}", cluster_name);
    let int_user_id: i32 = user_id.parse().unwrap();
    let ingress_path = format!("{}-wrk.{}", cluster_name, config.host);
    let workspace_name = format!("workspace-{}", cluster_name);
    let int_cluster_id = clusters::cluster_id(&pool, &int_user_id, &cluster_name)
        .await
        .unwrap();

    if !clusters::name_belongs_to(&pool, &int_user_id, &cluster_name)
        .await
        .unwrap()
    {
        return HttpResponse::NotFound().json(json!({"message": format!("Workspace cluster {} not found for uid {}", cluster_name, int_user_id)}));
    }

    info!(
        "cluster workspace to be created for cluster {}",
        cluster_name
    );

    if !utils::namevalid(&workspace_name) {
        return HttpResponse::ImATeapot().finish(); // this shouldn't ever happen
    }

    core_workspace_create(
        &kubeclient,
        &pool,
        &config.host,
        ingress_path.as_str(),
        &config.ingress_class,
        workspace_name.as_str(),
        cluster_name.as_str(),
        namespace.as_str(),
        &int_user_id,
        &int_cluster_id,
    )
    .await;

    HttpResponse::Ok().json(json!({"path": ingress_path}))
}

#[post("/api/workspaces/createtoken/{cluster_id}")]
pub async fn create_token_for_terminal(
    pool: web::Data<PgPool>,
    user: AuthUser,
    cluster_id: web::Path<i32>,
) -> HttpResponse {
    let token = Uuid::new_v4().to_string();
    let int_user_id: i32 = user.user_id.parse().unwrap();
    let int_cluster_id: i32 = cluster_id.into_inner();
    let created_at = chrono::Utc::now();

    if !clusters::id_belongs_to(&pool, &int_user_id, &int_cluster_id)
        .await
        .unwrap()
    {
        return HttpResponse::NotFound().json(json!({"message": format!("Workspace cluster {} not found for uid {}", int_cluster_id, int_user_id)}));
    }

    // add token to database
    workspaces::token_create(&pool, &token, &int_user_id, &int_cluster_id, &created_at)
        .await
        .expect("failed adding workspace token to db");

    HttpResponse::Ok().json(json!({"token": token}))
}

#[post("/api/workspaces/validate_token/{cluster_id}/{token}")]
pub async fn validate_terminal_access(
    pool: web::Data<PgPool>,
    path: web::Path<(i32, String)>,
) -> HttpResponse {
    println!("meow!");
    let (int_cluster_id, token) = path.into_inner();
    // let int_user_id: i32 = user.user_id.parse().unwrap();
    // let right_now = chrono::Utc::now();
    // the token is only valid if created in the last 10 seconds
    let valid_window = chrono::Utc::now() - chrono::Duration::seconds(60);

    // if !check_cluster_ownership(&pool, &int_cluster_id, None, Some(&int_cluster_id)).await {
    //     return HttpResponse::NotFound().json(json!({"message": format!("Workspace cluster {} not found for uid {}", int_cluster_id)}));
    // }

    let valid = sqlx::query("UPDATE workspace_tokens SET used=True WHERE token=$1 AND cluster_id=$2 AND used=False AND created_at>$3 RETURNING user_id")
        .bind(&token)
        .bind(int_cluster_id)
        .bind(valid_window)
        .fetch_optional(pool.get_ref())
        .await;

    match valid {
        Ok(Some(_)) => HttpResponse::Ok().finish(),
        Ok(None) => {
            info!(
                "Responding Unauthorized on cluster {} with token {}",
                int_cluster_id, token
            );
            HttpResponse::Unauthorized()
                .json(json!({"message": "Invalid, expired, or already used token"}))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
