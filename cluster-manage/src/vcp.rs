use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use std::collections::BTreeMap;
use kube::{Client};
use k3k_rs;
use k3k_rs::virtualclusterpolicy::{
    VirtualClusterPolicy, VirtualClusterPolicySpec, SyncSpec,
    SyncResourceSpec, LimitSpec, LimitsSpec, QuotaSpec, ScopeSelectorSpec
};

pub async fn create_default_vcp (
    kubeclient: &Client,
    clustername: &String,
    namespace: &String) -> bool {

    let vpc_name = format!("vpc-{}", clustername);

    let schema = VirtualClusterPolicy {
        metadata: kube::core::ObjectMeta {
            name: Some(vpc_name),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },

        spec: VirtualClusterPolicySpec {
            allowedMode: "shared".to_string(),
            // defaultNodeSelector: (),
            // defaultPriorityClass: (),
            // disableNetworkPolicy: (),
            // limit: (),
            podSecurityAdmissionLevel:  Some("restricted".to_string()),
            quota: Some(QuotaSpec {
                hard: Some(BTreeMap::from([
                    ("cpu".to_string(), IntOrString::String("500m".into())),
                    ("memory".to_string(), IntOrString::String("750m".into())),
                    ("storage".to_string(), IntOrString::String("4G".into())),
                ])),
               ..Default::default()
                // scopeSelector: (),
                // scopes: ()
            }),
            sync: Some(SyncSpec {
                ingresses: Some(SyncResourceSpec { enabled: true, selector: None }),
                ..Default::default()
            }),
            ..Default::default()
       }
    };

    k3k_rs::virtualclusterpolicy::create(kubeclient, &schema, namespace).await
        .expect(format!("Failed to create vcp for cluster {}", clustername).as_str());

    true
}
