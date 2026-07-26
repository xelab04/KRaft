import logging
import utils


def get_storage_reserved_cluster_longhorn(custom_api):
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
                    disk_status[disk].get('storageScheduled', '0')
                )

        return total_storage

    except Exception as e:
        logging.warning("Error getting Longhorn used storage: %s", e)
        return None

# get storage which has been reserved across cluster
def get_pv_claimed_storage(api_instance, custom_api):

    # attempt to get longhorn metrics
    longhorn_used_storage = get_storage_reserved_cluster_longhorn(custom_api)
    # if longhorn metrics are not none, use that
    if longhorn_used_storage:
        return longhorn_used_storage

    # otherwise grab all pvs and sum up the claimed storage
    returned_list_of_pvs = api_instance.list_persistent_volume(watch=False)

    total_claimed_storage = 0
    for persistent_volume in returned_list_of_pvs.items:
        capacity = persistent_volume.spec.capacity
        if capacity:
            total_claimed_storage += utils.convert_storage(
                str(capacity.get('storage', '0'))
            )

    return total_claimed_storage

# get storage reserved in namespace
def get_pvc_claimed_storage(api_instance, namespace):
    returned_list_of_pvcs = api_instance.list_namespaced_persistent_volume_claim(namespace=namespace)

    total_claimed_storage = 0
    for persistent_volume_claim in returned_list_of_pvcs.items:
        capacity = persistent_volume_claim.status.capacity
        if capacity:
            total_claimed_storage += utils.convert_storage(capacity.get('storage', '0'))

    return total_claimed_storage
