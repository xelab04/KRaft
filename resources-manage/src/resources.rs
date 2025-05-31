use actix_web::web::{Json, Path};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Serialize};

use k8s_openapi::api::core::v1::Pod;
// use k8s_openapi::api::metrics::v1beta1::PodMetrics;

use serde_json::json;
use tracing::*;

use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt, DynamicObject, GroupVersionKind},
    Client,
};
use kube::core::ApiResource;

use crate::util;

#[get("/get/resources/{ns}")]
pub async fn get(ns: web::Path<String>) -> HttpResponse {
    let namespace = ns.into_inner();

    // tracing_subscriber::fmt::init();

    let Ok(client) = Client::try_default().await else {
        return HttpResponse::Forbidden().finish();
    };


    // Manage pods
    let pods: Api<Pod> = Api::namespaced(client.clone(), &namespace);

    match pods.list(&Default::default()).await {
        Ok(pod_list) => {
            println!("{}", pod_list.items.len());
            for pod in pod_list {
                println!("{}", pod.metadata.name.unwrap_or_default());
            }
        },
        Err(e) => {
            println!("Error getting pods, {}", e);
            return HttpResponse::ImATeapot().finish();
        }
    };

    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let ar = ApiResource {
        group: "metrics.k8s.io".into(),
        version: "v1beta1".into(),
        api_version: "metrics.k8s.io/v1beta1".into(),
        kind: "PodMetrics".into(),
        plural: "pods".into(), // plural name of the resource
    };
    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), &namespace, &ar);

    // let pods_metrics: Api<PodMetrics> = Api::namespaced(client, &namespace);

    match api.list(&Default::default()).await {
        Ok(metrics_list) => {
            for m in metrics_list.items {
                println!("PodMetrics name: {}", m.name_any(), );

                if let Some(containers) = m.data.get("containers").and_then(|c| c.as_array()) {
                    for c in containers {
                        let name = c.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        let cpu = c
                            .get("usage")
                            .and_then(|u| u.get("cpu"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("n/a");

                        let memory = c
                            .get("usage")
                            .and_then(|u| u.get("memory"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("n/a");

                        println!("  Container: {}, CPU: {}, RAM: {}", name, cpu, memory);
                    }
                } else {
                    println!("  No containers found");
                }
            }
        }
        Err(_) => println!("Oops")
    }



    HttpResponse::Ok()
        .content_type("application/json")
        .json(serde_json::json!({"key": namespace}))
}






// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     tracing_subscriber::fmt::init();
//     let client = Client::try_default().await?;

//     // Manage pods
//     let pods: Api<Pod> = Api::default_namespaced(client);

//     // Verify we can get it
//     info!("Get Pod blog");
//     let p1cpy = pods.get("blog").await?;
//     if let Some(spec) = &p1cpy.spec {
//         info!("Got blog pod with containers: {:?}", spec.containers);
//         assert_eq!(spec.containers[0].name, "blog");
//     }

//     // Replace its spec
//     info!("Patch Pod blog");
//     let patch = json!({
//         "metadata": {
//             "resourceVersion": p1cpy.resource_version(),
//         },
//         "spec": {
//             "activeDeadlineSeconds": 5
//         }
//     });
//     let patchparams = PatchParams::default();
//     let p_patched = pods.patch("blog", &patchparams, &Patch::Merge(&patch)).await?;
//     assert_eq!(p_patched.spec.unwrap().active_deadline_seconds, Some(5));

//     let lp = ListParams::default().fields(&format!("metadata.name={}", "blog")); // only want results for our pod
//     for p in pods.list(&lp).await? {
//         info!("Found Pod: {}", p.name_any());
//     }

//     // Delete it
//     let dp = DeleteParams::default();
//     pods.delete("blog", &dp).await?.map_left(|pdel| {
//         assert_eq!(pdel.name_any(), "blog");
//         info!("Deleting blog pod started: {:?}", pdel);
//     });

//     Ok(())
// }
