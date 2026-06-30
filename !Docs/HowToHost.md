# KRaft
![written_in_rust badge](https://img.shields.io/badge/rust%F0%9F%8F%B3%EF%B8%8F%E2%80%8D%E2%9A%A7%EF%B8%8F-gray?label=written%20in&labelColor=orange)  [![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)  ![Website](https://img.shields.io/website?url=https%3A%2F%2Fkraftcloud.dev)

Oh hey, I didn't actually expect people to be interested in hosting their own version of KRaft. This is cool :3

This is the documentation for hosting KRaft on your own cluster. I have made the best effort to provide somewhat coherent and correct instructions. However, due to differences in hosting environments, your mileage may vary. I strongly urge you to report any discrepancies or issues you encounter as an issue.

## Dependencies
- Hardware: KRaft itself is not resource intensive, with only 50MB used across the frontend and core services. Each virtual cluster, however, uses approximately 400MB for the kube-server, kubelet, and CoreDNS.
- Kubernetes: The host cluster should be running an up-to-date version of K\*s. While development was done on a K3s cluster, there are no special requirements, so any distro should work.
- Ingress: While Gateway API is something I am contemplating, it is not yet fully supported in KRaft. This means having an ingress class, and in my case I have used Traefik. However, there are no custom annotations or resources, so again, any ingress class should work. Do note that you should avoid using Ingress Nginx as far as possible, firstly because it is end of life, but also because it does not [support multi-tenancy](https://kubernetes.github.io/ingress-nginx/faq/#multi-tenant-kubernetes).
- Storage: On my personal cluster, I have been using Longhorn. However, there are no hard requirements on storage class, meaning KRaft will work for Longhorn, Rook+Ceph, and even LocalPath. Appropriate measures must be made for volume snapshots and backups, without which the virtual clusters' etcd will be unrecoverable.
- IP: Only one public IP address needs to be pointed to the host cluster's ingress. All cluster and workload traffic is designed to pass through solely the ingress. Therefore, provision does not have to be made for NodePort or LoadBalancer IPs which are not exposed to the outside world. If you do not have a public IP on your existing infra, or are behind next-level ISP tomfoolery, then you are recommended to use [Towonel](https://towonel.dev/) on a VPS or share one.
- Certmanager: In order to issue certificates for securely accessing both the KRaft web services, and the cluster workspaces, Certmanager must be installed and KRaft must be configured with a cluster issuer.
- K3k: K3k is the project which allows running virtual clusters. It is a Rancher project, and can be installed through Helm. There is no hard dependency on K3k version for the time being; this will change in the future, but I am still developing KRaft and keeping it in line with the latest K3k version. Refer to the K3k repo [here](https://github.com/rancher/k3k/)

## Database
- KRaft works only with PostgreSQL databases and you are required to deploy your own. I would recommend Cloud Native Postgres, though you can also use a simple Postgres pod for testing purposes and if you do not value your own sanity when something breaks.
- The database connection details must then be provided to KRaft (obviously!)

## Deploying with Helm

Fun fact! This is the first time I have made a Helm chart! Expect issues!!
Everything regarding Helm is in the !Helm folder. You should be able to deploy it directly with Helm. I think? 

I use ArgoCD to deploy my workloads, so you can also just yoink the following yaml. Don't forget your database, and pay attention to the other configuration settings!!

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: kraft-helm
  namespace: argocd
spec:
  project: default
  source:
    repoURL: 'https://github.com/xelab04/KRaft.git'
    targetRevision: helm
    path: '!Helm/kraft'

    helm:
      values: |
        database:
          secret: database-secrets
        networking:
          clusterIssuer: issuer

  destination:
    server: 'https://kubernetes.default.svc'
    namespace: kraft-helm

  syncPolicy:
    syncOptions:
      - CreateNamespace=true
```
