mod backend
mod frontend

dev-up: dev-down build
	docker compose -f infra/docker-compose.yml up -d --build

dev-down:
	docker compose -f infra/docker-compose.yml down

dev-logs:
	docker compose -f infra/docker-compose.yml logs -f

dev-ca:
	docker compose -f infra/docker-compose.yml exec -T caddy cat /data/caddy/pki/authorities/local/root.crt > infra/dev-ca.crt
	@echo "Wrote infra/dev-ca.crt"

down: dev-down test-down

check: backend::check frontend::check

lint: backend::lint frontend::lint

pretty: backend::pretty frontend::pretty

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
	$compose up -d --wait gitea
	$compose exec -T -u git gitea gitea admin user create --admin --username "$GITEA_ADMIN_USER" --password "$GITEA_ADMIN_PASSWORD" --email ci@fortropp.test --must-change-password=false || true
	export GITEA_RUNNER_TOKEN=$($compose exec -T -u git gitea gitea actions generate-runner-token)
	$compose up -d gitea-runner

ci-down:
	docker compose -f infra/docker-compose.test.yml -f infra/docker-compose.ci.yml --env-file frontend/.env.test rm -sfv gitea gitea-runner

test: test-up backend::test frontend::test test-down
