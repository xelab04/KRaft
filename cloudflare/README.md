# CloudFlare Management
Uses CloudFlare API to create and manage DNS records for the endpoints for Kubernetes clusters.

## Needs
- CloudFlare account with API access
- Domain pointing to host cluster's LB IP to which new CNAME records will be created
- Kubernetes ingress controller and ingress resource

## Use
- Go to "manage endpoints"
- Click "create endpoint"
- In Kubernetes, create ingress with the obtained URL (eg: orange-egg.alexbissessur.dev)
- Visit the URL to access the service
