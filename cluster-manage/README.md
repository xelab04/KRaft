# Manage Clusters
Manage existing clusters created by the user

## Needs
- User ID/Token

## Use
- Checks DB for existing clusters
- Uses resource management to find used resources by each cluster
- Return list of clusters and resource used/reserved

## Use Create Cluster
- Send ID
- Send cluster name (optional)
- Uses helm/vcluster/k3k to deploy cluster
- Obtains a free nodeport and reserves it
- Retrieve kubeconfig
- Edit kubeconfig with the new IP + nodeport

## ToDo
- Fix JWT validation when Auth JWT is completed
- Exit on invalid JWT
- Add Postgres connection and `select where ID=user_id`
