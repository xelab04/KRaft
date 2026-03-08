.PHONY: frontend redeploy all db-init auth clustermanage podman resourcemanage
.SILENT:
VERSION := $(shell cat version.txt)

frontend:
	cd frontend && podman build . -t registry.alexbissessur.dev/kraft-frontend
	podman push registry.alexbissessur.dev/kraft-frontend

clustermanage:
	cd cluster-manage && podman build . -t registry.alexbissessur.dev/kraft-cluster-manage
	podman push registry.alexbissessur.dev/kraft-cluster-manage

resourcemanage:
	cd resource-manage && podman build . -t registry.alexbissessur.dev/kraft-resource-manage
	podman push registry.alexbissessur.dev/kraft-resource-manage

auth:
	cd auth && podman build . -t registry.alexbissessur.dev/kraft-auth
	podman push registry.alexbissessur.dev/kraft-auth

podman:
	cd frontend && podman build . -t registry.alexbissessur.dev/kraft-frontend
	podman push registry.alexbissessur.dev/kraft-frontend

	cd auth && podman build . -t registry.alexbissessur.dev/kraft-auth
	podman push registry.alexbissessur.dev/kraft-auth

	cd cluster-manage && podman build . -t registry.alexbissessur.dev/kraft-cluster-manage
	podman push registry.alexbissessur.dev/kraft-cluster-manage

	cd resource-manage && podman build . -t registry.alexbissessur.dev/kraft-resource-manage
	podman push registry.alexbissessur.dev/kraft-resource-manage

	# cd cloudflare-manage && podman build . -t registry.alexbissessur.dev/kraft-cloudflare-manage
	# podman push registry.alexbissessur.dev/kraft-cloudflare-manage

prod:
	cd frontend && podman build . -t xelab04/kraft-frontend:$(VERSION)
	podman push xelab04/kraft-frontend:$(VERSION)

	cd auth && podman build . -t xelab04/kraft-auth:$(VERSION)
	podman push xelab04/kraft-auth:$(VERSION)

	cd cluster-manage && podman build . -t xelab04/kraft-cluster-manage:$(VERSION)
	podman push xelab04/kraft-cluster-manage:$(VERSION)

	cd resource-manage && podman build . -t xelab04/kraft-resource-manage:$(VERSION)
	podman push xelab04/kraft-resource-manage:$(VERSION)
