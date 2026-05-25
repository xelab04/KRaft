use crate::Controllers::utils;
use crate::Models::Resources::{cluster_resources, namespace_resources};
use actix_web::HttpResponse;
use actix_web::web;
use actix_web::web::Path;
use k8s_openapi::api::core::v1::Node;
use kube::ResourceExt;
use kube::{
    Api, Client,
    core::{ApiResource, DynamicObject},
};
use serde_json::json;

async fn get_pod_use(client: &Client, namespace: &str) -> namespace_resources {
    // let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "pods");
    // let ar = ApiResource::from_gvk(&gvk);

    let ar = ApiResource {
        group: "metrics.k8s.io".into(),
        version: "v1beta1".into(),
        kind: "PodMetrics".into(),
        api_version: "metrics.k8s.io/v1beta1".into(),
        plural: "pods".into(),
    };

    let api_handler: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);
    let pod_metrics = api_handler.list(&Default::default()).await.unwrap();

    let mut total_pods_cpu_use: i32 = 0;
    let mut total_pods_mem_use: i32 = 0;
    for pod in pod_metrics.items {
        let containers_array = pod.data["containers"].as_array();
        if let Some(containers) = containers_array {
            for container in containers {
                let name = container["name"].as_str().unwrap_or("unknown");
                let cpu = utils::convert_cpu(container["usage"]["cpu"].as_str().unwrap_or("0"));
                let memory =
                    utils::convert_memory(container["usage"]["memory"].as_str().unwrap_or("0"));

                println!(
                    "pod={} container={} cpu={} memory={}",
                    pod.name_any(),
                    name,
                    cpu,
                    memory
                );

                total_pods_cpu_use += cpu;
                total_pods_mem_use += memory;
            }
        }
    }

    return namespace_resources {
        cpu: total_pods_cpu_use,
        memory: total_pods_mem_use,
        storage: 0,
    };
}

#[actix_web::get("/resources/ns/{namespace}")]
async fn get_namespace_use(kubeclient: web::Data<Client>, namespace: Path<String>) -> HttpResponse {
    println!("meow!");
    let pod_use = get_pod_use(&kubeclient, &namespace).await;

    return HttpResponse::Ok().json(pod_use);
}

async fn get_node_use(client: &Client) -> cluster_resources {
    let ar = ApiResource {
        group: "metrics.k8s.io".into(),
        version: "v1beta1".into(),
        kind: "NodeMetrics".into(),
        api_version: "metrics.k8s.io/v1beta1".into(),
        plural: "nodes".into(),
    };

    let api_handler: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
    let node_metrics = api_handler.list(&Default::default()).await.unwrap();

    let mut total_nodes_cpu_use: i32 = 0;
    let mut total_nodes_mem_use: i32 = 0;
    for node in node_metrics.items {
        let name = node.name_any();
        let cpu = utils::convert_cpu(node.data["usage"]["cpu"].as_str().unwrap_or("0"));
        let memory = utils::convert_memory(node.data["usage"]["memory"].as_str().unwrap_or("0"));

        total_nodes_cpu_use += cpu;
        total_nodes_mem_use += memory;
    }

    return cluster_resources {
        cpu: total_nodes_cpu_use,
        memory: total_nodes_mem_use,
        storage: 100,
    };
}

async fn get_node_capacity(client: &Client) -> cluster_resources {
    let nodes: Api<Node> = Api::all(client.clone());
    let node_list = nodes.list(&Default::default()).await.unwrap();

    let total_cpu = node_list
        .items
        .iter()
        .filter_map(|n| {
            n.status
                .as_ref()
                .unwrap()
                .capacity
                .as_ref()
                .unwrap()
                .get("cpu")
        })
        .map(|c| utils::convert_cpu(&c.0))
        .sum();

    let total_memory = node_list
        .items
        .iter()
        .filter_map(|n| {
            n.status
                .as_ref()
                .unwrap()
                .capacity
                .as_ref()
                .unwrap()
                .get("memory")
        })
        .map(|c| utils::convert_memory(&c.0))
        .sum();

    return cluster_resources {
        cpu: total_cpu,
        memory: total_memory,
        storage: 100,
    };
}

#[actix_web::get("/resources/cluster")]
async fn get_cluster_use(kubeclient: web::Data<Client>) -> HttpResponse {
    let node_use = get_node_use(&kubeclient).await;
    let node_capacity = get_node_capacity(&kubeclient).await;

    let return_json = json!({
        "status": "success",
        "storage": {
            "claimed": node_use.storage,
            "allocatable": node_capacity.storage
        },
        "cpu": {
            "total": node_capacity.cpu,
            "claimed": node_use.cpu
        },
        "memory": {
            "total": node_capacity.memory,
            "claimed": node_use.memory
        }
    });

    return HttpResponse::Ok().json(return_json);
}
