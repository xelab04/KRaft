.PHONY: frontend redeploy all db-init auth clustermanage podman resourcemanage
.SILENT:
VERSION := $(shell cat version.txt)

frontend:
	cd frontend && podman build . -t registry.alexbissessur.dev/kraft-frontend
	podman push registry.alexbissessur.dev/kraft-frontend

kraft:
	cd kraft-core && podman build . -t registry.alexbissessur.dev/kraft-core
	podman push registry.alexbissessur.dev/kraft-core

proxy:
	cd workspace/sidecar && podman build . -t registry.alexbissessur.dev/kraft-workspace-proxy
	podman push registry.alexbissessur.dev/kraft-workspace-proxy

wkspc:
	cd workspace && podman build . -t registry.alexbissessur.dev/kraft-workspace
	podman push registry.alexbissessur.dev/kraft-workspace

wkspc-nix:
	cd workspace/nix && nix-build main.nix && podman load < result && podman tag localhost/kraft-workspace:latest ghcr.io/xelab04/kraft-workspace:latest
	# registry.alexbissessur.dev/kraft-workspace:latest
	podman push ghcr.io/xelab04/kraft-workspace:latest

podman:
	cd frontend && podman build . -t registry.alexbissessur.dev/kraft-frontend
	podman push registry.alexbissessur.dev/kraft-frontend

	cd kraft && podman build . -t registry.alexbissessur.dev/kraft-core
	podman push registry.alexbissessur.dev/kraft-core

prod:
	cd frontend && podman build . -t xelab04/kraft-frontend:$(VERSION)
	podman push xelab04/kraft-frontend:$(VERSION)

	cd kraft && podman build . -t xelab04/kraft-core:$(VERSION)
	podman push xelab04/kraft-core:$(VERSION)
