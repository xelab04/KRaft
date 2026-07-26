"""
1. Get clusters from database
2. For each cluster
2.1 Get resources used
2.2 Add to database
2.3 Update averages
"""
import mysql.connector
from dotenv import load_dotenv
import logging
import os
import sys
from kubernetes import client, config

import pods, nodes, storage, utils

load_dotenv()


def get_db_cursor():
    DB_CONFIG = {
        "host": os.environ["DATABASE_URL"],
        "port": os.environ["DATABASE_PORT"],
        "user": os.environ["DATABASE_USER"],
        "password": os.environ["DATABASE_PASSWORD"],
        "database": os.environ["DATABASE_DATABASE"],
    }

    conn = mysql.connector.connect(**DB_CONFIG)
    cursor = conn.cursor()
    return conn, cursor


def get_clusters():
    conn, cursor = get_db_cursor()

    cursor.execute("SELECT cluster_name FROM clusters")
    cluster_names = [row[0] for row in cursor.fetchall()]

    cursor.close()
    conn.close()
    return cluster_names


def cluster_usage(cluster_name):
    cluster_ns = "k3k-" + cluster_name

    config.load_incluster_config()
    api_instance = client.CoreV1Api()
    custom_api = client.CustomObjectsApi()

    compute = pods.get_pod_use(api_instance, custom_api, cluster_ns)
    sto = storage.get_pvc_claimed_storage(api_instance, cluster_ns)
    total_cpu = compute["total_cpu"]
    total_memory = compute["total_memory"]

    conn, cursor = get_db_cursor()
    cursor.execute(
        "INSERT INTO cluster_usage (cluster_name, cpu, memory, storage, recorded_at) "
        "VALUES (%s, %s, %s, %s, NOW())",
        (cluster_name, total_cpu, total_memory, sto),
    )
    conn.commit()
    cursor.close()
    conn.close()

    logging.info("Recorded usage for %s: cpu=%s, memory=%s, storage=%s", cluster_name, total_cpu, total_memory, sto)


def main():
    clusters = get_clusters()
    for cluster in clusters:
        cluster_usage(cluster)


if __name__ == "__main__":
    if not all(k in os.environ for k in ("DATABASE_URL", "DATABASE_PORT", "DATABASE_USER", "DATABASE_PASSWORD", "DATABASE_DATABASE")):
        logging.error("Missing required environment variables: DATABASE_URL, DATABASE_PORT, DATABASE_USER, DATABASE_PASSWORD, DATABASE_DATABASE")
        sys.exit(1)
    main()
