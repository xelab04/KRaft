use std::collections::BTreeMap;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use actix_web::web;
use actix_web::web::Json;
use actix_web::{HttpRequest, HttpResponse};

use sqlx;
use sqlx::PgPool;

use crate::class;
use crate::validatename;
use crate::AppConfig;

use crate::class::{AuthUser, Cluster, ClusterCreateForm};

use kube::{
    api::{Api, PostParams},
    core::{DynamicObject, GroupVersionKind, ApiResource},
    Client,
};
use serde_json::json;


pub async fn ingressroute(client: &Client, cluster_name: &str, namespace: &str, host: &str) {

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
                    "match": format!("Host(`{}`)", host),
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
                "targetPort": 7681
            }],
            "type": "ClusterIP"
        }
    });

    let pp = PostParams::default();
    let ingressroute: DynamicObject = serde_json::from_value(svc).unwrap();

    let _created = services.create(&pp, &ingressroute).await.unwrap();
}

pub async fn statefulset(client: &Client, cluster_name: &str, namespace: &str) {

    // define CRD type
    let gvk = GroupVersionKind::gvk("apps", "v1", "StatefulSet");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let statefulsets: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);

    // json cause im lazy
    let ss = json!({
        "apiVersion": "v1",
        "kind": "Service",
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
                    "containers": [{
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
                    }],
                    "volumes": [{
                        "name": "kubeconfig",
                        "secret": {
                            "secretName": "k3k-k-1-easterhegg-kubeconfig",
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
    let ingressroute: DynamicObject = serde_json::from_value(ss).unwrap();

    let _created = statefulsets.create(&pp, &ingressroute).await.unwrap();
}

#[post("/api/create/workspaces")]
pub async fn create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    kubeclient: web::Data<Client>,
    config: web::Data<AppConfig>,
    user: AuthUser,
    Json(cluster): Json<ClusterCreateForm>,
) -> HttpResponse {

    let user_id = user.user_id;
    let cluster_name = format!("k-{}-{}", user_id, cluster.name);
    let namespace = format!("k3k-{}", cluster_name);
    let int_user_id: i32 = user_id.parse().unwrap();

    // check the cluster exists and belongs to that user
    let cluster_belongs_to_user: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM clusters WHERE user_id = $1 AND cluster_name = $2)")
        .bind(&int_user_id)
        .bind(&cluster_name)
        .fetch_one(pool.get_ref())
        .await
        .expect("Failed to fetch cluster count");

    if !cluster_belongs_to_user {
        return HttpResponse::NotFound().json(json!({"message": "Workspace cluster not found"}));
    }

    let workspace_name = format!("workspace-{}", cluster_name);
    if !validatename::namevalid(&workspace_name) {
        return HttpResponse::ImATeapot().finish(); // this shouldn't ever happen
    }

    statefulset(&kubeclient, cluster_name.as_str(), namespace.as_str()).await;

    service(&kubeclient, cluster_name.as_str(), namespace.as_str()).await;

    ingressroute(&kubeclient, cluster_name.as_str(), namespace.as_str(), &config.host).await;

    // let int_user_id = user_id.parse::<i32>().unwrap();
    sqlx::query("INSERT INTO workspaces (workspace_name, user_id) VALUES ($1, $2)")
        .bind(workspace_name)
        .bind(int_user_id)
        .execute(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Ok().finish()
}
