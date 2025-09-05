import utils

def get_pv_claimed_storage(api_instance):
    returned_list_of_pvs = api_instance.list_persistent_volume(watch=False)

    total_claimed_storage = 0
    for persistent_volume in returned_list_of_pvs.items:
        # pprint(persistent_volume)

        try:
            # this caters for longhorn.
            # apologies to the RookCeph people
            number_of_replicas = int(persistent_volume.spec.csi.volume_attributes['numberOfReplicas'])
        except KeyError:
            number_of_replicas = 1

        total_claimed_storage += utils.convert_storage(persistent_volume.spec.capacity['storage']) * number_of_replicas

    return total_claimed_storage

def get_pvc_claimed_storage(api_instance, namespace):
    returned_list_of_pvcs = api_instance.list_namespaced_persistent_volume_claim(namespace=namespace)

    total_claimed_storage = 0
    for persistent_volume_claim in returned_list_of_pvcs.items:
        # pprint(persistent_volume_claim)
        total_claimed_storage += utils.convert_storage(persistent_volume_claim.status.capacity['storage'])

    return total_claimed_storage
