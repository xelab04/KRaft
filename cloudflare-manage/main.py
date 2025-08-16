#!/usr/bin/env python3
import os
import requests
import sys
from flask import Flask

app = Flask(__name__)

API_TOKEN   = os.environ.get("CF_API_TOKEN")
ZONE_ID     = os.environ.get("CF_ZONE_ID")
TARGET      = os.environ.get("CF_CNAME_TARGET")
PROXIED     = os.environ.get("CF_CNAME_PROXIED", "true").lower() == "true"

if not all([API_TOKEN, ZONE_ID, TARGET]):
    sys.exit("❌ Missing one or more required environment variables")

@app.route("/api/cloudflare/create/<name>")
def add(name):
    BASE_URL = f"https://api.cloudflare.com/client/v4/zones/{ZONE_ID}/dns_records"
    HEADERS = {"Authorization": f"Bearer {API_TOKEN}", "Content-Type": "application/json"}

    # check if record exists
    resp = requests.get(BASE_URL, headers=HEADERS, params={"type": "CNAME", "name": name})
    resp.raise_for_status()
    records = resp.json().get("result", [])

    payload = {
        "type": "CNAME",
        "name": name,
        "content": TARGET,
        "ttl": 1,
        "proxied": PROXIED
    }

    if records:
        record_id = records[0]["id"]
        print(f"🔄 Updating CNAME record {name} -> {TARGET}")
        r = requests.put(f"{BASE_URL}/{record_id}", headers=HEADERS, json=payload)
    else:
        print(f"➕ Creating CNAME record {name} -> {TARGET}")
        r = requests.post(BASE_URL, headers=HEADERS, json=payload)

    print(f"Status: {r.status_code}")
    print(r.json())

    if r.status_code >= 200 and r.status_code < 300:
        return "success"

    return "failure"


@app.route("/api/cloudflare/delete")
def delete():
    pass

app.run(host="0.0.0.0", port=5000, debug=False)
