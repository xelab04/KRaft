
# KRaft

When you want to learn Kubernetes or have a quick lab, maybe for workshops, it becomes chaos. Users on Windows, or Mac, or Linux, and half a thousand ways to run Kubernetes in a way conducive to learning makes it unpleasant. Rancher made Hobby Farm, which spins up VMs on cloud service providers, and their implementation is a lot more thorough than KRaft hopes to be.


## What is KRaft?

KRaft provides a very opinionated but fully-contained platform which runs on top of a host Kubernetes cluster, and spins up virtual clusters for each person who wants one. You can use this for workshops and training sessions. Or you can try it out as "cloud service provider" for you and your friends to share resources over a lab.
## Components

- Create cluster
- View virtual clusters
- Integration with CloudFlare DNS for creating URLs for services
## Contributing

Contributions are always welcome!
Just check the issues page and let me know if you'll take one on. However, please communicate with me when adding large features.

One limitation on all contributions - I will automatically reject any PRs with code written by LLMs and forward you to [justforfunnoreally.dev](https://justforfunnoreally.dev/) because you evidently need to learn to have fun with programming again.


## Structure

Behold, a very ugly way of explaining the different parts I want.
![Excalidraw Screenshot](https://files.alexbissessur.dev/CRaft.png)
