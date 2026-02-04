import utils
from pprint import pprint


def get_storage_reserved_cluster_longhorn(custom_api):
    try:
        longhorn_nodes = custom_api.list_cluster_custom_object(
            group="longhorn.io",
            version="v1beta2",
            plural="nodes",
            watch=False
        )

        total_storage = 0
        for node in longhorn_nodes['items']:
            for disk in node['status']['diskStatus']:
                total_storage += utils.convert_storage(node['status']['diskStatus'][disk]['storageScheduled'])

        return total_storage

    except Exception as e:
        print(f"Error getting Longhorn used storage: {e}")
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
        # pprint(persistent_volume)
        total_claimed_storage += utils.convert_storage(str(persistent_volume.spec.capacity['storage']))

    return total_claimed_storage

# get storage reserved in namespace
def get_pvc_claimed_storage(api_instance, namespace):
    returned_list_of_pvcs = api_instance.list_namespaced_persistent_volume_claim(namespace=namespace)

    total_claimed_storage = 0
    for persistent_volume_claim in returned_list_of_pvcs.items:
        # pprint(persistent_volume_claim)
        total_claimed_storage += utils.convert_storage(persistent_volume_claim.status.capacity['storage'])

    return total_claimed_storage
