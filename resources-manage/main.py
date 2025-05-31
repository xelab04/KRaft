from kubernetes import client, config
from flask import Flask, send_file, jsonify

app = Flask(__name__)


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

@app.route('/get/resources/<namespace>', methods=['GET'])
def get_resources(namespace):
    config.load_kube_config()
    api_instance = client.CoreV1Api()

    ret = api_instance.list_namespaced_pod(namespace=namespace, watch=False)

    total_cpu, total_memory = 0, 0

    for pod in ret.items:
        for container in pod.spec.containers:
            cpu, memory = 0, 0
            if(container.resources.requests):
                if container.resources.requests.get('cpu'):
                    cpu = convert_cpu(container.resources.requests['cpu'])
                if container.resources.requests.get('memory'):
                    memory = convert_memory(container.resources.requests['memory'])

            total_cpu += cpu
            total_memory += memory

    return {
        "cpu": total_cpu,
        "memory": total_memory
    }

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
