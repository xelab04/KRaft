use actix_web::HttpResponse;
use actix_web::web;
use actix_web::web::Path;
use kube::{
    Api, Client,
    core::{ApiResource, DynamicObject, GroupVersionKind},
};

async fn get_pod_use(client: &Client, namespace: &str) {
    let gvk = GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "pods");
    let ar = ApiResource::from_gvk(&gvk);

    let api_handler: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
    let pod_metrics = api_handler.list(&Default::default()).await.unwrap();

    for pod in pod_metrics.items {
        print!("{:?}", pod);
    }
}

#[actix_web::get("/resources/{namespace}")]
async fn get_namespace_use(kubeclient: web::Data<Client>, namespace: Path<String>) -> HttpResponse {
    get_pod_use(&kubeclient, &namespace).await;

    return HttpResponse::Ok().finish();
}
