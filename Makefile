.PHONY: frontend redeploy all db-init auth clustermanage docker resourcemanage
.SILENT:
VERSION := $(shell cat version.txt)

frontend:
	cd frontend && docker build . -t registry.alexbissessur.dev/kraft-frontend
	docker push registry.alexbissessur.dev/kraft-frontend

clustermanage:
	cd cluster-manage && docker build . -t registry.alexbissessur.dev/kraft-cluster-manage
	docker push registry.alexbissessur.dev/kraft-cluster-manage

resourcemanage:
	cd resource-manage && docker build . -t registry.alexbissessur.dev/kraft-resource-manage
	docker push registry.alexbissessur.dev/kraft-resource-manage

auth:
	cd auth && docker build . -t registry.alexbissessur.dev/kraft-auth
	docker push registry.alexbissessur.dev/kraft-auth

docker:
	cd frontend && docker build . -t registry.alexbissessur.dev/kraft-frontend
	docker push registry.alexbissessur.dev/kraft-frontend

	cd auth && docker build . -t registry.alexbissessur.dev/kraft-auth
	docker push registry.alexbissessur.dev/kraft-auth

	cd cluster-manage && docker build . -t registry.alexbissessur.dev/kraft-cluster-manage
	docker push registry.alexbissessur.dev/kraft-cluster-manage

	cd resource-manage && docker build . -t registry.alexbissessur.dev/kraft-resource-manage
	docker push registry.alexbissessur.dev/kraft-resource-manage

	# cd cloudflare-manage && docker build . -t registry.alexbissessur.dev/kraft-cloudflare-manage
	# docker push registry.alexbissessur.dev/kraft-cloudflare-manage

prod:
	cd frontend && docker build . -t xelab04/kraft-frontend:$(VERSION)
	docker push xelab04/kraft-frontend:$(VERSION)

	cd auth && docker build . -t xelab04/kraft-auth:$(VERSION)
	docker push xelab04/kraft-auth:$(VERSION)

	cd cluster-manage && docker build . -t xelab04/kraft-cluster-manage:$(VERSION)
	docker push xelab04/kraft-cluster-manage:$(VERSION)

	cd resource-manage && docker build . -t xelab04/kraft-resource-manage:$(VERSION)
	docker push xelab04/kraft-resource-manage:$(VERSION)
