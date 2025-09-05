"""
1. Get clusters from database
2. For each database
2.1 Get resources used
2.2 Add to database
2.3 Update averages
2.4
"""
import mysql.connector
from dotenv import load_dotenv
import os
import sys
from pprint import pprint
import time
from kubernetes import client, config


import pods, nodes, storage, utils

load_dotenv()

def get_db_cursor():
    DB_CONFIG = {
        "host": DATABASE_URL,
        "port": DATABASE_PORT,
        "user": DATABASE_USER,
        "password": DATABASE_PASSWORD,
        "database": DATABASE_DATABASE
    }

    conn = mysql.connector.connect(**DB_CONFIG)
    cursor = conn.cursor()

    return conn, cursor

try:
    DATABASE_URL=os.environ["DATABASE_URL"]
    DATABASE_PORT=os.environ["DATABASE_PORT"]
    DATABASE_USER=os.environ["DATABASE_USER"]
    DATABASE_PASSWORD=os.environ["DATABASE_PASSWORD"]
    DATABASE_DATABASE=os.environ["DATABASE_DATABASE"]
except KeyError:
    print("Missing env vars, proceeding with defaults. DO NOT DO THIS IN PRODUCTION")
    DATABASE_URL="127.0.0.1"
    DATABASE_PORT="3306"
    DATABASE_USER="root"
    DATABASE_PASSWORD="password"
    DATABASE_DATABASE="kraft"
    try:
        conn, cursor = get_db_cursor()
        conn.close()
    except :
        print("Failed to connect to database")
        time.sleep(5)
        sys.exit(1)

def get_clusters():
    conn, cursor = get_db_cursor()

    cursor.execute("SELECT cluster_name FROM clusters")
    clusters = cursor.fetchall()

    clusters = [i[0] for i in clusters]
    return clusters

def cluster_usage(cluster_name):
    cluster_ns = "k3k-" + cluster_name

    config.load_kube_config()
    api_instance = client.CoreV1Api()
    custom_api = client.CustomObjectsApi()


    compute = pods.get_pod_use(api_instance, custom_api, cluster_ns)
    sto = storage.get_pvc_claimed_storage(api_instance, cluster_ns)
    total_cpu, total_memory = compute["total_cpu"], compute["total_memory"]

    # now i put this in a table
    # and alter the number of cpu/ram hours



def main():
    clusters = get_clusters()

    pprint(clusters)

if __name__ == "__main__":
    main()
