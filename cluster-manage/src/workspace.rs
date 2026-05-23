use std::collections::BTreeMap;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use actix_web::web;
use actix_web::web::Json;
use actix_web::{HttpRequest, HttpResponse};
use serde::{self, Serialize, Deserialize};

use log::{info, error};
use uuid::Uuid;
use chrono;

use sqlx;
use sqlx::PgPool;

use crate::class;
use crate::AppConfig;

use crate::class::{AuthUser, Cluster, ClusterCreateForm};

use kube::{
    api::{Api, PostParams},
    core::{DynamicObject, GroupVersionKind, ApiResource},
    Client,
};
use serde_json::json;


pub async fn ingress(client: &Client, cluster_name: &str, namespace: &str, ingress_path: &str) {

    let gvk = GroupVersionKind::gvk("networking.k8s.io", "v1", "Ingress");
    let ar = ApiResource::from_gvk(&gvk);

    let ingress_handler: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);

    let ingress = json!({
        "apiVersion": "networking.k8s.io/v1",
        "kind": "Ingress",
        "metadata": {
            "name": format!("workspace-{}",cluster_name),
            "namespace": namespace,
            "annotations": [{
                "cert-manager.io/cluster-issuer": "prod-issuer"
            }]
        },
        "spec": {
            // TODO: make ingress class dynamic
            "ingressClassName": "traefik",
            "tls": [{
                "hosts": [ingress_path],
                "secretName": format!("{}-tls", cluster_name)
            }],
            "rules": [{
                "host": ingress_path,
                "http": {
                    "paths": [{
                        "path": "/",
                        "pathType": "prefix",
                        "backend": {
                            "service": {
                                "name": format!("workspace-{}",cluster_name),
                                    "port": 8080
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

pub async fn ingressroute(client: &Client, cluster_name: &str, namespace: &str, ingress_path: &str) {

    // define CRD type
    let gvk = GroupVersionKind::gvk("traefik.io", "v1alpha1", "IngressRoute");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let ingress_routes: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);

    // json cause im lazy
    let ingressroute = json!({
        "apiVersion": "traefik.io/v1alpha1",
        "kind": "IngressRoute",
        "metadata": {
            "name": format!("workspace-{}",cluster_name),
            "namespace": namespace
        },
        "spec": {
            "entryPoints": ["websecure", "web"],
            "routes": [
                {
                    "kind": "Rule",
                    "match": format!("Host(`{}`)", ingress_path),
                    "services": [
                        {
                            "name": format!("workspace-{}",cluster_name),
                            "port": 8080
                        }
                    ]
                }
            ],
            "tls": { "certResolver": "le" }
        }
    });

    let pp = PostParams::default();
    let ingressroute: DynamicObject = serde_json::from_value(ingressroute).unwrap();

    let _created = ingress_routes.create(&pp, &ingressroute).await.unwrap();
}

pub async fn service(client: &Client, cluster_name: &str, namespace: &str) {

    // define CRD type
    let gvk = GroupVersionKind::gvk("", "v1", "Service");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let services: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);

    // json cause im lazy
    let svc = json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": format!("workspace-{}",cluster_name),
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

    let _created = services.create(&pp, &ingressroute).await.unwrap();
}

pub async fn statefulset(client: &Client, cluster_name: &str, namespace: &str, cluster_id: &i32, appconfig: &AppConfig) {

    // define CRD type
    let gvk = GroupVersionKind::gvk("apps", "v1", "StatefulSet");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let statefulsets: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);

    // json cause im lazy
    let ss = json!({
        "apiVersion": "apps/v1",
        "kind": "StatefulSet",
        "metadata": {
            "name": "workspace",
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
                            "image": "registry.alexbissessur.dev/kraft-workspace-proxy:latest",
                            "imagePullPolicy": "Always",
                            "env": [
                                {
                                    "name": "HOST",
                                    "value": appconfig.host
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
                                    "memory": "50M"
                                }
                            },
                            "securityContext": {
                                "allowPrivilegeEscalation": false,
                                "runAsUser": 1000
                            }
                        },
                        {
                            "name": "workspace",
                            "image": "registry.alexbissessur.dev/kraft-workspace:latest",
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

    let _created = statefulsets.create(&pp, &statefulset).await.unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct WorkspaceCreate {
    pub name: String
}


pub async fn check_cluster_ownership(pool: &web::Data<PgPool>, user_id: &i32, cluster_name: Option<&String>, cluster_id: Option<&i32>) -> bool {
    let cluster_belongs_to_user: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = $1 AND (cluster_name = $2 OR cluster_id = $3))")
        .bind(&user_id)
        .bind(&cluster_name)
        .bind(&cluster_id)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to fetch cluster count");

    return cluster_belongs_to_user
}

pub async fn get_cluster_id_from_name(pool: &web::Data<PgPool>, user_id: &i32, cluster_name: &str) -> i32 {
    let int_cluster_id = sqlx::query_scalar("SELECT cluster_id FROM clusters WHERE user_id = $1 AND cluster_name = $2")
        .bind(user_id)
        .bind(cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to get cluster id");

    return int_cluster_id;
}

#[post("/api/create/workspaces")]
pub async fn create(
    req: HttpRequest,
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
    let int_cluster_id = get_cluster_id_from_name(&pool, &int_user_id, &cluster_name).await;

    if !check_cluster_ownership(&pool, &int_user_id, Some(&cluster_name), None).await {
        return HttpResponse::NotFound().json(json!({"message": format!("Workspace cluster {} not found for uid {}", cluster_name, int_user_id)}));
    }

    // check the cluster exists and belongs to that user
    // let cluster_belongs_to_user: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = $1 AND cluster_name = $2)")
    //     .bind(&int_user_id)
    //     .bind(&cluster_name)
    //     .fetch_one(pool.get_ref())
    //     .await
    //     .expect("Failed to fetch cluster count");

    // if !cluster_belongs_to_user {
    //     return HttpResponse::NotFound().json(json!({"message": format!("Workspace cluster {} not found for uid {}", cluster_name, int_user_id)}));
    // }

    // check if an existing workspace... exists
    let cluster_workspace_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM workspaces WHERE user_id = $1 AND cluster_name = $2)")
        .bind(&int_user_id)
        .bind(&cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to check if cluster workspace exists");

    if cluster_workspace_exists {
        info!("cluster workspace already exists for cluster {}", cluster_name);
        println!("cluster workspace already exists for cluster {}", cluster_name);
        return HttpResponse::Ok().json(json!({"path": ingress_path}));
    }

    println!("cluster workspace to be created for cluster {}", cluster_name);

    let workspace_name = format!("workspace-{}", cluster_name);
    if !class::namevalid(&workspace_name) {
        return HttpResponse::ImATeapot().finish(); // this shouldn't ever happen
    }

    statefulset(&kubeclient, cluster_name.as_str(), namespace.as_str(), &int_cluster_id, &config).await;

    service(&kubeclient, cluster_name.as_str(), namespace.as_str()).await;

    ingress(&kubeclient, cluster_name.as_str(), namespace.as_str(), &ingress_path).await;

    // let int_user_id = user_id.parse::<i32>().unwrap();
    sqlx::query("INSERT INTO workspaces (workspace_name, cluster_name, user_id) VALUES ($1, $2, $3)")
        .bind(&workspace_name)
        .bind(&cluster_name)
        .bind(&int_user_id)
        .execute(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Ok().json(json!({"path": ingress_path}))
}


#[post("/api/workspaces/createtoken/{cluster_id}")]
pub async fn create_token_for_terminal(
    pool: web::Data<PgPool>,
    user: AuthUser,
    cluster_id: web::Path<i32>
) -> HttpResponse {

    let token = Uuid::new_v4().to_string();
    let int_user_id: i32 = user.user_id.parse().unwrap();
    let int_cluster_id: i32 = cluster_id.into_inner();
    let created_at = chrono::Utc::now();

    if !check_cluster_ownership(&pool, &int_user_id, None, Some(&int_cluster_id)).await {
        return HttpResponse::NotFound().json(json!({"message": format!("Workspace cluster {} not found for uid {}", int_cluster_id, int_user_id)}));
    }

    // add token to database
    let _r = sqlx::query("INSERT INTO workspace_tokens (token, user_id, cluster_id, created_at, used) VALUES ($1, $2, $3, $4, $5)")
        .bind(&token)
        .bind(&int_user_id)
        .bind(&int_cluster_id)
        .bind(&created_at)
        .bind(false)
        .execute(pool.get_ref())
        .await
        .unwrap();

    return HttpResponse::Ok().json(json!({"token": token}));
}

#[post("/api/workspaces/validatetoken/{cluster_id}/{token}")]
pub async fn validate_terminal_access(
    pool: web::Data<PgPool>,
    // user: AuthUser,
    // cluster_id: web::Path<i32>,
    // token: web::Path<String>
    path: web::Path<(i32, String)>
) -> HttpResponse {

    let (int_cluster_id, token) = path.into_inner();
    // let int_user_id: i32 = user.user_id.parse().unwrap();
    // let right_now = chrono::Utc::now();
    // the token is only valid if created in the last 10 seconds
    let valid_window = chrono::Utc::now() - chrono::Duration::seconds(60);

    // if !check_cluster_ownership(&pool, &int_cluster_id, None, Some(&int_cluster_id)).await {
    //     return HttpResponse::NotFound().json(json!({"message": format!("Workspace cluster {} not found for uid {}", int_cluster_id)}));
    // }

    let valid = sqlx::query("UPDATE workspace_tokens SET used=True WHERE token=$1 AND cluster_id=$2 AND used=False AND created_at>$3 RETURNING user_id")
        .bind(token)
        .bind(int_cluster_id)
        .bind(valid_window)
        .fetch_optional(pool.get_ref())
        .await;

    match valid {
        Ok(Some(_)) => HttpResponse::Ok().finish(),
        Ok(None) => HttpResponse::Unauthorized().json(json!({"message": "Invalid, expired, or already used token"})),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}
