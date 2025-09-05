import utils

def get_pod_use(api_instance, custom_api, namespace):

    try:
        metrics = custom_api.list_namespaced_custom_object(
            group="metrics.k8s.io",
            version="v1beta1",
            namespace=namespace,
            plural="pods"
        )

        total_cpu, total_memory = 0, 0

        for pod_metric in metrics['items']:
            for container in pod_metric['containers']:
                cpu_usage = container['usage'].get('cpu', '0')
                memory_usage = container['usage'].get('memory', '0')

                cpu = utils.convert_cpu(cpu_usage)
                memory = utils.convert_memory(memory_usage)

                total_cpu += cpu
                total_memory += memory

    except Exception as e:
        print(f"Error getting metrics: {e}")

        return get_pod_requests(api_instance, namespace)
        # Meh I'm sure requests is a fair comparison


    return {
        "total_cpu": total_cpu,
        "total_memory": total_memory
    }

def get_pod_requests(api_instance, namespace):
    ret = api_instance.list_namespaced_pod(namespace=namespace, watch=False)
    total_cpu, total_memory = 0, 0

    for pod in ret.items:
        for container in pod.spec.containers:
            cpu, memory = 0, 0
            if(container.resources.requests):
                if container.resources.requests.get('cpu'):
                    cpu = utils.convert_cpu(container.resources.requests['cpu'])
                if container.resources.requests.get('memory'):
                    memory = utils.convert_memory(container.resources.requests['memory'])

            total_cpu += cpu
            total_memory += memory

    return {
        "total_cpu": total_cpu,
        "total_memory": total_memory
    }
