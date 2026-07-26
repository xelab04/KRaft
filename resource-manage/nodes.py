import logging
import utils

def get_node_compute_capacity(api_instance):
    nodes = api_instance.list_node(watch=False)

    total_cpu_capacity = sum([
        utils.convert_cpu(node.status.capacity.get('cpu', '0'))
        for node in nodes.items if node.status.capacity
    ])

    total_mem_capacity = sum([
        utils.convert_memory(node.status.capacity.get('memory', '0'))
        for node in nodes.items if node.status.capacity
    ])

    return {
        "total_cpu": total_cpu_capacity,
        "total_memory": total_mem_capacity
    }

def get_node_use(custom_api):

    try:
        # Get node metrics from metrics.k8s.io API
        metrics = custom_api.list_cluster_custom_object(
            group="metrics.k8s.io",
            version="v1beta1",
            plural="nodes"
        )

        total_cpu = sum([utils.convert_cpu(node.get('usage', {}).get('cpu', 0)) for node in metrics.get('items', [])])
        total_memory = sum([utils.convert_memory(node.get('usage', {}).get('memory', 0)) for node in metrics.get('items', [])])

    except Exception as e:
        logging.warning("Error getting node metrics: %s", e)

        return {
            "total_cpu": 0,
            "total_memory": 0
        }

    return {
        "total_cpu": total_cpu,
        "total_memory": total_memory
    }

def get_node_storage_longhorn(custom_api):
    try:
        longhorn_nodes = custom_api.list_cluster_custom_object(
            group="longhorn.io",
            version="v1beta2",
            plural="nodes",
            watch=False
        )

        total_storage = 0

        for node in longhorn_nodes.get('items', []):
            disk_status = node.get('status', {}).get('diskStatus', {})
            for disk in disk_status:
                total_storage += utils.convert_storage(
                    disk_status[disk].get('storageMaximum', '0')
                )

        return total_storage

    except Exception as e:
        logging.warning("Error getting total storage: %s", e)
        return None

def get_allocatable_node_storage(api_instance, custom_api):
    returned_list_of_nodes = api_instance.list_node(watch=False)

    total_allocatable_storage = get_node_storage_longhorn(custom_api)
    if total_allocatable_storage:
        return total_allocatable_storage

    total_allocatable_storage = sum([
        utils.convert_storage(node.status.allocatable.get('ephemeral-storage', '0'))
        for node in returned_list_of_nodes.items
        if node.status.allocatable
    ])

    return total_allocatable_storage
