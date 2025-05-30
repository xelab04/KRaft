from kubernetes import client, config

def convert_cpu(cpu):
    if cpu.endswith("m"):
        cpu = int(cpu.strip("m"))
    elif cpu.endswith("cpu"):
        cpu = int(cpu)
    else:
        raise ValueError("Invalid CPU value")
    return cpu

def main():
    config.load_kube_config()
    api_instance = client.CoreV1Api()
    print("Listing pods with their IPs:")
    ret = api_instance.list_namespaced_pod(namespace="caddy", watch=False)
    for pod in ret.items:
        for container in pod.spec.containers:
            print(container.resources.requests['cpu'])
            print(container.resources.requests['memory'])
        print("%s\t%s\t%s" % (pod.status.pod_ip, pod.metadata.namespace, pod.metadata.name))
    print("Done")


main()
