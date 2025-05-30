# Create A Cluster
It creates a new virtual cluster

## Needs
- User ID/Token

## Use
- Send ID
- Send cluster name (optional)
- Uses helm/vcluster/k3k to deploy cluster
- Obtains a free nodeport and reserves it
- Retrieve kubeconfig
- Edit kubeconfig with the new IP + nodeport
