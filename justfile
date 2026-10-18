mod backend
mod frontend
mod worker

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

test: test-up backend::test worker::proofs frontend::test test-down
