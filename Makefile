.PHONY: frontend redeploy all db-init admin-ui admin-api auth-ui auth-api
.SILENT:


docker:
	cd frontend && docker build . -t registry.alexbissessur.dev/kraft-frontend
	docker push registry.alexbissessur.dev/kraft-frontend

	cd auth && docker build . -t registry.alexbissessur.dev/kraft-auth
	docker push registry.alexbissessur.dev/kraft-auth

	cd cluster-manage && docker build . -t registry.alexbissessur.dev/kraft-cluster-manage
	docker push registry.alexbissessur.dev/kraft-cluster-manage

	cd cloudflare-manage && docker build . -t registry.alexbissessur.dev/kraft-cloudflare-manage
	docker push registry.alexbissessur.dev/kraft-cloudflare-manage

	cd resources-manage && docker build . -t registry.alexbissessur.dev/kraft-resources-manage
	docker push registry.alexbissessur.dev/kraft-resources-manage
