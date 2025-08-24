.PHONY: frontend redeploy all db-init auth clustermanage docker
.SILENT:

frontend:
	cd frontend && docker build . -t registry.alexbissessur.dev/kraft-frontend
	docker push registry.alexbissessur.dev/kraft-frontend

clustermanage:
	cd cluster-manage && docker build . -t registry.alexbissessur.dev/kraft-cluster-manage
	docker push registry.alexbissessur.dev/kraft-cluster-manage

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

	# cd cloudflare-manage && docker build . -t registry.alexbissessur.dev/kraft-cloudflare-manage
	# docker push registry.alexbissessur.dev/kraft-cloudflare-manage

	# cd resources-manage && docker build . -t registry.alexbissessur.dev/kraft-resources-manage
	# docker push registry.alexbissessur.dev/kraft-resources-manage
