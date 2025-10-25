# KRaft
[![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)

This is the documentation for hosting KRaft on your own cluster. I have made the best effort to provide somewhat coherent and correct instructions. However, due to differences in hosting environments, your mileage may vary. I strongly urge you to report any discrepancies or issues you encounter as an issue.

## Host Cluster
- The host cluster should be running on reasonable hardware, for starters.
- Theoretically, any Kubernetes distribution should work fine, though I would recommend K3s as that is what I have been developing on.
- Have Longhorn installed. Using another storageclass is possible, though this will interfere with the resource graph on the home screen.
- Have a single ingressClass - Traefik. KRaft will share the same ingressClass with the guest clusters. Nginx Ingress does not [support multi-tenancy](https://kubernetes.github.io/ingress-nginx/faq/#multi-tenant-kubernetes).
- Have a public IP pointed to the ingress's service.

## Database
KRaft uses a simple database for storing users and cluster data. I have yet to add an initcontainer to setup the database, though the script can be found in the database service directory.

## Deploying with Helm
Fun fact! This is the first time I have made a Helm chart! Expect issues!!
Everything regarding Helm is in the !Helm folder. You should be able to deploy it directly with Helm. I think?

Let me brief you on the values file.

| Value  | Possible Values | Purpose |
| ------------- | ------------- |
| betaCode  | string | Registration requires a beta code. You can leave this as an empty string.  |
| environment  | PROD / LOCAL | If not set to PROD, this disables some of the authentication features. You should leave this to PROD.  |
| host | string | The hostname of the KRaft instance. It is used for ingresses, as well as for exposing guest control planes. |
| database.hosted.enabled | boolean | For dev purposes, I put a simple db deployment. This is tolerable but probably not ideal for production. |
| database.hosted.image | string | The image used for the hosted database. Must be MySQL or MariaDB. |
| database.hosted.size | string | The size of the PVC used for the hosted db. Defaults to 4Gi. |
| database.* | string | The remaining settings are just typical database configuration. |
