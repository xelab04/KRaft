import utils
from pprint import pprint

def get_node_capacity(api_instance):
    nodes = api_instance.list_node(watch=False)

    total_cpu_capacity = sum([utils.convert_cpu(node.status.capacity.get('cpu', '0')) for node in nodes.items])
    total_mem_capacity = sum([utils.convert_memory(node.status.capacity.get('memory', '0')) for node in nodes.items])

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

        total_cpu = sum([utils.convert_cpu(node['usage'].get('cpu', 0)) for node in metrics['items']])
        total_memory = sum([utils.convert_memory(node['usage'].get('memory', 0)) for node in metrics['items']])

    except Exception as e:
        print(f"Error getting metrics: {e}")

        return {
            "total_cpu": 0,
            "total_memory": 0
        }

    return {
        "total_cpu": total_cpu,
        "total_memory": total_memory
    }

def get_allocatable_node_storage(api_instance, custom_api):
    returned_list_of_nodes = api_instance.list_node(watch=False)

    try:
        longhorn_nodes = custom_api.list_cluster_custom_object(
            group="longhorn.io",
            version="v1beta2",
            plural="nodes",
            watch=False
        )

        total_storage = 0

        for node in longhorn_nodes['items']:
            # reserved = sum([utils.convert_storage(disk.storageReserved) for disk in node.spec.disks])
            for disk in node['status']['diskStatus']:
                # pprint(disk)
                total_storage += utils.convert_storage(node['status']['diskStatus'][disk]['storageMaximum'])

            # total_storage_on_node = sum([ )

        return total_storage

    except Exception as e:
        print(f"Error getting total storage: {e}")


    total_allocatable_storage = sum([utils.convert_storage(node.status.allocatable['ephemeral-storage']) for node in returned_list_of_nodes.items])

    return total_allocatable_storage
