.PHONY: frontend redeploy all db-init auth clustermanage podman resourcemanage
.SILENT:
VERSION := $(shell cat version.txt)

frontend:
	cd frontend && podman build . -t registry.alexbissessur.dev/kraft-frontend
	podman push registry.alexbissessur.dev/kraft-frontend

resourcemanage:
	cd resource-manage && podman build . -t registry.alexbissessur.dev/kraft-resource-manage
	podman push registry.alexbissessur.dev/kraft-resource-manage

kraft:
	cd kraft && podman build . -t registry.alexbissessur.dev/kraft-core
	podman push registry.alexbissessur.dev/kraft-core

workspace-proxy:
	cd workspace/sidecar && podman build . -t registry.alexbissessur.dev/kraft-workspace-proxy
	podman push registry.alexbissessur.dev/kraft-workspace-proxy

podman:
	cd frontend && podman build . -t registry.alexbissessur.dev/kraft-frontend
	podman push registry.alexbissessur.dev/kraft-frontend

	cd kraft && podman build . -t registry.alexbissessur.dev/kraft-core
	podman push registry.alexbissessur.dev/kraft-core

	cd resource-manage && podman build . -t registry.alexbissessur.dev/kraft-resource-manage
	podman push registry.alexbissessur.dev/kraft-resource-manage

prod:
	cd frontend && podman build . -t xelab04/kraft-frontend:$(VERSION)
	podman push xelab04/kraft-frontend:$(VERSION)

	cd kraft && podman build . -t xelab04/kraft-core:$(VERSION)
	podman push xelab04/kraft-core:$(VERSION)

	cd resource-manage && podman build . -t xelab04/kraft-resource-manage:$(VERSION)
	podman push xelab04/kraft-resource-manage:$(VERSION)
