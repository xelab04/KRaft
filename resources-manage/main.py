from kubernetes import client, config

def convert_cpu(cpu):
    if cpu.endswith("m"):
        cpu = int(cpu.strip("m"))
    else:
        try:
            cpu = int(cpu) * 1000
        except ValueError:
            raise ValueError("Invalid CPU value")
    return cpu

def convert_memory(mem):
    if mem.endswith("Mi"):
        mem = int(mem.strip("Mi"))
    elif mem.endswith("Gi"):
        # multiplying by 1024 is ugly
        mem = int(mem.strip("Gi")) * 1024
    else:
        raise ValueError("Invalid memory value")
    return mem

def main():
    config.load_kube_config()
    api_instance = client.CoreV1Api()
    print("Listing pods with their IPs:")
    ret = api_instance.list_namespaced_pod(namespace="caddy", watch=False)
    for pod in ret.items:
        for container in pod.spec.containers:
            cpu, memory = 0, 0
            if(container.resources.requests):
                if container.resources.requests.get('cpu'):
                    cpu = convert_cpu(container.resources.requests['cpu'])
                if container.resources.requests.get('memory'):
                    memory = convert_memory(container.resources.requests['memory'])
            print(cpu, memory)
            # print(container.resources.requests['cpu'])
            # print(container.resources.requests['memory'])
        print("%s\t%s\t%s" % (pod.status.pod_ip, pod.metadata.namespace, pod.metadata.name))
    print("Done")


main()
