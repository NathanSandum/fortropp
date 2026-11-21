mod backend
mod frontend
mod worker

dev-up: dev-down build
	op run --env-file infra/backend-dev.env.tpl -- docker compose -f infra/docker-compose.yml up -d --build
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

prod-rewrite-routes: ssh-key-loaded
	#!/usr/bin/env bash
	set -euo pipefail
	host=$(cd infra/tofu && op run --env-file .env.tpl -- tofu output -raw deploy_host)
	ssh "root@$host" 'cd /opt/fortropp && docker compose exec -T backend ./rewrite_routes'

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
			["CLOUDFLARE_HOSTNAMES_ZONE_ID", "cloudflare_hostnames_zone_id"],
			["ORIGIN_IPV4", "deploy_host"],
			["TURNSTILE_SITEKEY", "turnstile_sitekey"],
			["TURNSTILE_SECRET", "turnstile_secret"]
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

ci-cache-secret:
	#!/usr/bin/env bash
	set -euo pipefail
	endpoint=$(cd infra/tofu && op run --no-masking --env-file .env.tpl -- tofu output -raw s3_endpoint)
	read -rp "Access Key ID: " key_id
	read -rsp "Secret Access Key: " secret_key
	echo
	printf 'SCCACHE_BUCKET=fortropp-ci-cache\nSCCACHE_ENDPOINT=%s\nSCCACHE_REGION=auto\nAWS_ACCESS_KEY_ID=%s\nAWS_SECRET_ACCESS_KEY=%s\n' "$endpoint" "$key_id" "$secret_key" \
		| gh secret set SCCACHE_ENV --repo NathanSandum/fortropp
	echo "Updated SCCACHE_ENV in NathanSandum/fortropp"

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

prod-hostnames +hostnames: whoami
	#!/usr/bin/env bash
	set -euo pipefail
	zone=$(cd infra/tofu && op run --no-masking --env-file .env.tpl -- tofu output -raw cloudflare_hostnames_zone_id)
	token=$(op read op://fortropp/backend-prod/CLOUDFLARE_API_TOKEN)
	for hostname in {{hostnames}}; do
		curl -sS -G --data-urlencode "hostname.exact=$hostname" \
			-H @<(printf 'Authorization: Bearer %s\n' "$token") \
			"https://api.cloudflare.com/client/v4/zones/$zone/custom_hostnames" \
			| jq -r --arg hostname "$hostname" '
				if .success | not then
					"\($hostname): lookup failed: \(.errors | map("\(.message) (\(.code))") | join("; "))"
				elif .result == [] then
					"\($hostname): not registered"
				else
					.result[0]
					| "\($hostname): hostname \(.status), certificate \(.ssl.status // "none")",
						"  hostname error: \((.verification_errors // [])[])",
						"  certificate error: \((.ssl.validation_errors // [])[].message)"
				end'
	done

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

dev-stripe-listen:
	stripe listen --forward-to https://localhost/api/stripe/webhook --skip-verify --events invoice.paid,invoice.payment_failed,invoice.payment_action_required,invoice.finalization_failed

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

test-quick: backend::test worker::proofs test-services frontend::test-unit

rollback-rehearsal previous recovery="" old_ui_skip="no test has this name" open_tabs="yes": test-up
	#!/usr/bin/env bash
	set -euo pipefail
	caffeinate -i -w $$ &
	compose="docker compose -f infra/docker-compose.test.yml --env-file frontend/.env.test"
	worktree=$(mktemp -d)
	old_frontend=$(mktemp)
	old_backend=$(mktemp)
	old_both=$(mktemp)
	survivor=$(mktemp)
	cleanup() {
		$compose down -v
		git worktree remove --force "$worktree"
		rm -f "$old_frontend" "$old_backend" "$old_both" "$survivor" "$survivor.storage"
	}
	trap cleanup EXIT
	step() {
		printf '\n=== %s\n' "$1"
	}
	step "Building {{previous}}"
	git fetch -q origin --tags
	git worktree add -q --detach "$worktree" "{{previous}}"
	git -C "$worktree" submodule update --init -q backend frontend worker
	docker build -q -t "fortropp-rehearsal-backend:{{previous}}" \
		--build-arg CARGO_FEATURES=test-routes \
		--build-context site-config="$worktree/worker/site-config" \
		--build-context metadata="$worktree" \
		"$worktree/backend"
	ln -s "$PWD/frontend/node_modules" "$worktree/frontend/node_modules"
	(cd "$worktree/frontend" && npm run build)
	printf 'services:\n  caddy:\n    volumes:\n      - %s/frontend/build:/srv\n' "$worktree" > "$old_frontend"
	printf 'services:\n  backend:\n    image: fortropp-rehearsal-backend:%s\n' "{{previous}}" > "$old_backend"
	cat "$old_backend" > "$old_both"
	cat "$old_frontend" | tail -n +2 >> "$old_both"
	e2e() {
		npx playwright test --retries=1 "$@"
	}
	full_e2e() {
		e2e --grep-invert @exclusive && e2e --grep @exclusive --workers=1
	}
	survivor_check() {
		(cd frontend && REHEARSAL_STEP="$1" REHEARSAL_STATE="$survivor" e2e rehearsal_pinned_preview)
	}
	restart_worker() {
		$compose restart worker
		$compose up -d --no-build --no-deps --wait worker
		$compose restart backend
	}
	(cd frontend && npx playwright install)
	step "New version, full e2e"
	(cd frontend && full_e2e)
	step "Pin a version on the preview link"
	survivor_check pin
	if [ "{{open_tabs}}" = "yes" ]; then
		step "Open tabs: {{previous}} frontend on the new backend"
		$compose -f "$old_frontend" up -d --no-build --no-deps --wait caddy
		(cd frontend && e2e publish_ui versions.e2e preview.e2e site_config_editor --grep-invert "{{old_ui_skip}}")
	fi
	if [ -n "{{recovery}}" ]; then
		$compose exec -T postgres psql -v ON_ERROR_STOP=1 -U fortropp -d fortropp < "{{recovery}}"
	fi
	if [ "{{open_tabs}}" = "yes" ]; then
		step "Open tabs after a rollback: new frontend on the {{previous}} backend"
		$compose -f "$old_backend" up -d --no-build --no-deps --wait backend caddy
		(cd "$worktree/frontend" && e2e publish_ui versions.e2e preview.e2e site_config_editor)
	fi
	step "Rolled back to {{previous}}, full e2e from that tag"
	restart_worker
	$compose -f "$old_both" up -d --no-build --no-deps --wait backend caddy
	survivor_check rolled-back
	(cd "$worktree/frontend" && full_e2e)
	step "Rolled forward again"
	$compose up -d --no-build --no-deps --wait backend caddy
	survivor_check rolled-forward
	(cd frontend && e2e publish_ui versions.e2e preview.e2e site_config_editor serving.e2e)
	step "Rollback to {{previous}} and back holds"

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
	released=$(git ls-remote --tags --refs origin | cut -f2)
	base="v$(date +%Y.%m.%d)"
	tag="$base"
	n=1
	while grep -qxF "refs/tags/$tag" <<< "$released"; do
		n=$((n + 1))
		tag="$base.$n"
	done
	git push -q origin "$commit:refs/tags/$tag"
	echo "Released $tag"
