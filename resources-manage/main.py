from kubernetes import client, config
from flask import Flask, send_file, jsonify
from pprint import pprint

import pods
import nodes
import storage

app = Flask(__name__)


# For the cluster view page
@app.route('/get/resources/<namespace>', methods=['GET'])
def get_resources(namespace):
    config.load_kube_config()
    api_instance = client.CoreV1Api()
    custom_api = client.CustomObjectsApi()

    compute = pods.get_pod_use(api_instance, custom_api, namespace)

    # One issue here is that the storage isn't multiplied by number of replicas
    sto = storage.get_pvc_claimed_storage(api_instance, namespace)

    total_cpu, total_memory = compute["total_cpu"], compute["total_memory"]

    return {
        "cpu": total_cpu,
        "memory": total_memory,
        "storage": sto
    }

# For the homepage to see cluster resource usage
@app.route('/get/cluster/resources', methods=["GET"])
def get_cluster_resources():
    config.load_kube_config()
    api_instance = client.CoreV1Api()
    custom_api = client.CustomObjectsApi()

    # storage
    pvc_claimed_storage = storage.get_pv_claimed_storage(api_instance, custom_api)
    allocatable_node_storage = nodes.get_allocatable_node_storage(api_instance, custom_api)

    # compute
    node_use = nodes.get_node_use(custom_api)
    used_cpu = node_use["total_cpu"]
    used_memory = node_use["total_memory"]

    node_capacity = nodes.get_node_capacity(api_instance)
    total_cpu = node_capacity["total_cpu"]
    total_memory = node_capacity["total_memory"]

    return jsonify({
        "status": "success",
        "storage": {
            "claimed": pvc_claimed_storage,
            "allocatable": allocatable_node_storage
        },
        "cpu": {
            "total": total_cpu,
            "claimed": used_cpu
        },
        "memory": {
            "total": total_memory,
            "claimed": used_memory
        }
    })



if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
