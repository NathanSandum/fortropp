mod backend
mod frontend
mod worker

dev-up: dev-down build
	docker compose -f infra/docker-compose.yml up -d --build
	just dev-dns

tofu := "cd infra/tofu && op run --env-file .env.tpl -- tofu"

prod-up:
	{{tofu}} apply

prod-plan:
	{{tofu}} plan

prod-init:
	{{tofu}} init

prod-ssh-key: whoami
	op read "op://fortropp/fortropp-backend-key/private_key?ssh-format=openssh" | ssh-add -t 900 -

ssh-key-loaded:
	@ssh-add -l > /dev/null 2>&1 || { echo "No key in ssh-agent: run just prod-ssh-key first"; exit 1; }

prod-cloud-init: ssh-key-loaded
	#!/usr/bin/env bash
	set -euo pipefail
	host=$(cd infra/tofu && op run --env-file .env.tpl -- tofu output -raw deploy_host)
	ssh -o StrictHostKeyChecking=accept-new "root@$host" cloud-init status --wait

prod-db-setup: ssh-key-loaded
	#!/usr/bin/env bash
	set -euo pipefail
	outputs=$(cd infra/tofu && op run --no-masking --env-file .env.tpl -- tofu output -json)
	host=$(printf '%s' "$outputs" | jq -r '.deploy_host.value')
	admin_url=$(printf '%s' "$outputs" | jq -r '.database_admin_url.value // error("missing output database_admin_url")')
	printf '%s\nGRANT USAGE, CREATE ON SCHEMA public TO fortropp;\n' "$admin_url" \
		| ssh "root@$host" 'read -r url && exec docker run --rm -i -e "DATABASE_URL=$url" postgres:18 sh -c "exec psql \"\$DATABASE_URL\" -v ON_ERROR_STOP=1"'

prod-secrets:
	#!/usr/bin/env bash
	set -euo pipefail
	repo=NathanSandum/fortropp
	outputs=$(cd infra/tofu && op run --no-masking --env-file .env.tpl -- tofu output -json)
	output() {
		printf '%s' "$outputs" | jq -r --arg name "$1" '.[$name].value // error("missing output \($name)")'
	}
	from_tofu=$(printf '%s' "$outputs" | jq -r '
		[
			["DATABASE_URL", "database_url"],
			["SESSION_KEY", "session_key"],
			["DOMAIN_VERIFICATION_KEY", "domain_verification_key"],
			["S3_ENDPOINT", "s3_endpoint"],
			["S3_PUBLIC_ORIGIN", "s3_endpoint"],
			["KV_API_URL", "kv_api_url"],
			["KV_NAMESPACE_ID", "kv_namespace_id"],
			["CLOUDFLARE_HOSTNAMES_ZONE_ID", "cloudflare_hostnames_zone_id"]
		][] as [$env, $name]
		| "\($env)=\(.[$name].value // error("missing output \($name)"))"')
	from_op=$(op inject -i infra/production/backend-prod.env.tpl)
	certificate=$(output origin_certificate)
	private_key=$(output origin_private_key)
	deploy_host=$(output deploy_host)
	deploy_key=$(output deploy_private_key)
	known_hosts=$(ssh-keyscan -t ed25519 "$deploy_host" 2>/dev/null)
	if [ -z "$known_hosts" ]; then
		echo "No ed25519 host key from $deploy_host; is the droplet up?"
		exit 1
	fi
	printf '%s\n%s\n' "$from_op" "$from_tofu" | gh secret set BACKEND_ENV --repo "$repo"
	printf '%s\n' "$certificate" | gh secret set ORIGIN_CERTIFICATE --repo "$repo"
	printf '%s\n' "$private_key" | gh secret set ORIGIN_PRIVATE_KEY --repo "$repo"
	printf '%s' "$deploy_host" | gh secret set DEPLOY_HOST --repo "$repo"
	printf '%s\n' "$deploy_key" | gh secret set DEPLOY_SSH_KEY --repo "$repo"
	printf '%s\n' "$known_hosts" | gh secret set DEPLOY_KNOWN_HOSTS --repo "$repo"
	echo "Updated BACKEND_ENV, ORIGIN_*, DEPLOY_* in $repo"

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
	op run --env-file infra/worker-deploy.env.tpl -- docker run --rm --entrypoint wrangler \
		-e CLOUDFLARE_API_TOKEN -e CLOUDFLARE_ACCOUNT_ID \
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

whoami:
	@op whoami > /dev/null 2>&1 || { echo "No 1Password session: run op signin first"; exit 1; }

deploy version: whoami
	#!/usr/bin/env bash
	set -euo pipefail
	repo=NathanSandum/fortropp
	built=$(gh run list --repo "$repo" --workflow Release --branch "{{version}}" --limit 1 --json conclusion --jq '.[0].conclusion // ""')
	if [ "$built" != "success" ]; then
		echo "{{version}} has no successful Release run (got: ${built:-none})"
		exit 1
	fi
	read -r -p "Deploy {{version}} to production? [y/N] " answer
	if [ "$answer" != "y" ]; then
		echo "Not deployed"
		exit 1
	fi
	gh workflow run Deploy --repo "$repo" -f version="{{version}}"
	sleep 5
	run=$(gh run list --repo "$repo" --workflow Deploy --limit 1 --json databaseId --jq '.[0].databaseId')
	gh run watch "$run" --repo "$repo" --exit-status

release: whoami
	#!/usr/bin/env bash
	set -euo pipefail
	published() {
		for repo in . backend frontend worker; do
			git -C "$repo" fetch -q origin
			if [ -n "$(git -C "$repo" status --porcelain)" ]; then
				echo "$repo has uncommitted or untracked changes"
				exit 1
			fi
			if ! git -C "$repo" merge-base --is-ancestor HEAD origin/HEAD; then
				echo "$repo has commits not on origin's default branch"
				exit 1
			fi
		done
	}
	published
	commit=$(git rev-parse HEAD)
	just test
	published
	if [ "$(git rev-parse HEAD)" != "$commit" ]; then
		echo "HEAD moved during the tests"
		exit 1
	fi
	git fetch -q --tags origin
	base="v$(date +%Y.%m.%d)"
	tag="$base"
	n=1
	while git rev-parse -q --verify "refs/tags/$tag" >/dev/null; do
		n=$((n + 1))
		tag="$base.$n"
	done
	git tag -a "$tag" -m "$tag" "$commit"
	git push -q origin "$tag"
	echo "Released $tag"
