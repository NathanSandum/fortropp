mod backend
mod frontend
mod worker

dev-up: dev-down build
	docker compose -f infra/docker-compose.yml up -d --build
	just dev-dns

tofu := "cd infra/tofu && set -a && . ./.env && set +a && tofu"

prod-up:
	{{tofu}} apply

prod-down:
	{{tofu}} destroy \
		-target=digitalocean_project.fortropp \
		-target=digitalocean_droplet.backend \
		-target=digitalocean_database_cluster.postgres \
		-target=digitalocean_firewall.backend \
		-target=digitalocean_vpc.production \
		-target=digitalocean_tag.backend \
		-target=digitalocean_ssh_key.operator

worker-deploy:
	docker compose -f infra/docker-compose.yml build worker
	set -a && . infra/tofu/.env && set +a && docker run --rm --entrypoint wrangler \
		-e CLOUDFLARE_API_TOKEN="$WORKER_DEPLOY_TOKEN" \
		-e CLOUDFLARE_ACCOUNT_ID="$TF_VAR_cloudflare_account_id" \
		infra-worker deploy --env=""

cloudflare-contract commit:
	#!/usr/bin/env bash
	set -euo pipefail
	spec=$(mktemp)
	trap 'rm -f "$spec"' EXIT
	curl -sfL -o "$spec" "https://raw.githubusercontent.com/cloudflare/api-schemas/{{commit}}/openapi.json"
	mkdir -p infra/test-services/contract
	cd infra/test-services && cargo run -q --bin extract_cloudflare_contract -- \
		"$spec" {{commit}} contract/cloudflare-custom-hostnames.json

dev_pdns := "http://localhost:8082/api/v1/servers/localhost"
dev_pdns_key := "fortropp-dev-pdns"

[private]
dev-zone zone:
	#!/usr/bin/env bash
	set -euo pipefail
	for _ in $(seq 60); do
		curl -sf -H "X-API-Key: {{dev_pdns_key}}" "{{dev_pdns}}/zones" >/dev/null && break
		sleep 1
	done
	status=$(curl -s -o /dev/null -w '%{http_code}' -H "X-API-Key: {{dev_pdns_key}}" "{{dev_pdns}}/zones/{{zone}}.")
	if [[ "$status" == 404 ]]; then
		curl -sf -X POST -H "X-API-Key: {{dev_pdns_key}}" -H 'Content-Type: application/json' \
			-d '{"name":"{{zone}}.","kind":"Native","nameservers":[]}' "{{dev_pdns}}/zones" >/dev/null
	fi

[private]
dev-record zone name type content:
	#!/usr/bin/env bash
	set -euo pipefail
	content='{{content}}'
	body=$(printf '{"rrsets":[{"name":"%s.","type":"%s","ttl":60,"changetype":"REPLACE","records":[{"content":"%s","disabled":false}]}]}' \
		'{{name}}' '{{type}}' "${content//\"/\\\"}")
	curl -sf -X PATCH -H "X-API-Key: {{dev_pdns_key}}" -H 'Content-Type: application/json' \
		-d "$body" "{{dev_pdns}}/zones/{{zone}}." >/dev/null

dev-dns: (dev-zone "fortropp.test")
	just dev-record fortropp.test fortropp.test A 127.0.0.1
	just dev-record fortropp.test '*.fortropp.test' A 127.0.0.1

dev-domain zone: (dev-zone zone)
	just dev-record {{zone}} {{zone}} ALIAS edge.fortropp.test.
	just dev-record {{zone}} www.{{zone}} CNAME edge.fortropp.test.

dev-proof zone name token: (dev-zone zone)
	just dev-record {{zone}} _fortropp-challenge.{{name}} TXT '"{{token}}"'

dev-down:
	docker compose -f infra/docker-compose.yml down

dev-logs:
	docker compose -f infra/docker-compose.yml logs -f

dev-ca:
	docker compose -f infra/docker-compose.yml exec -T caddy cat /data/caddy/pki/authorities/local/root.crt > infra/dev-ca.crt
	@echo "Wrote infra/dev-ca.crt"

down: dev-down test-down

check: backend::check frontend::check worker::check

lint: backend::lint frontend::lint worker::lint

pretty: backend::pretty frontend::pretty worker::pretty

build: backend::build frontend::build

test-up: down frontend::build
	docker compose -f infra/docker-compose.test.yml --env-file frontend/.env.test up -d --build --wait

test-down:
	docker compose -f infra/docker-compose.test.yml -f infra/docker-compose.ci.yml --env-file frontend/.env.test down -v

ci-up:
	#!/usr/bin/env bash
	set -euo pipefail
	compose="docker compose -f infra/docker-compose.test.yml -f infra/docker-compose.ci.yml --env-file frontend/.env.test"
	set -a; source frontend/.env.test; set +a
	$compose up -d --wait forgejo
	$compose exec -T -u git forgejo forgejo admin user create --admin --username "$FORGEJO_ADMIN_USER" --password "$FORGEJO_ADMIN_PASSWORD" --email ci@fortropp.test --must-change-password=false || true
	export FORGEJO_RUNNER_TOKEN=$($compose exec -T -u git forgejo forgejo actions generate-runner-token)
	$compose up -d forgejo-runner

ci-down:
	docker compose -f infra/docker-compose.test.yml -f infra/docker-compose.ci.yml --env-file frontend/.env.test rm -sfv forgejo forgejo-runner

test-services:
	cd infra/test-services && cargo clippy --all-targets -q && cargo test -q

test: test-up backend::test worker::proofs test-services frontend::test
	just test-down
