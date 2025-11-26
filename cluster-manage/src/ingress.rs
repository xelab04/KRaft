use kube::{
    api::{Api, PostParams, ResourceExt},
    core::{DynamicObject, GroupVersionKind, ApiResource},
    Client,
};
use serde_json::json;

pub async fn traefik(client: &Client, cluster_name: &String, namespace: &String, host: &str, n: usize) -> bool {

    // define CRD type
    let gvk = GroupVersionKind::gvk("traefik.io", "v1alpha1", "IngressRouteTCP");
    let ar = ApiResource::from_gvk(&gvk);

    // api
    let ingress_routes: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace.as_str(), &ar);

    // json cause im lazy
    let ingressroute = json!({
        "apiVersion": "traefik.io/v1alpha1",
        "kind": "IngressRouteTCP",
        "metadata": {
            "name": format!("api-svr-{}-{}-rt",cluster_name,n),
            "namespace": namespace
        },
        "spec": {
            "entryPoints": ["kraft"],
            "routes": [
                {
                    "match": format!("HostSNI(`{}`)", host),
                    "services": [
                        {
                            "name": format!("k3k-{}-service",cluster_name),
                            "port": 443
                        }
                    ]
                }
            ],
            "tls": { "passthrough": true }
        }
    });

    let pp = PostParams::default();
    let ingressroute: DynamicObject = serde_json::from_value(ingressroute).unwrap();

    let created = ingress_routes.create(&pp, &ingressroute).await.unwrap();

    true
}
