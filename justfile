mod backend
mod frontend

dev-up:
	docker compose -f infra/docker-compose.yml up -d --build

dev-down:
	docker compose -f infra/docker-compose.yml down

dev-logs:
	docker compose -f infra/docker-compose.yml logs -f

check: backend::check frontend::check

lint: backend::lint frontend::lint

pretty: backend::pretty frontend::pretty

build: backend::build frontend::build

test-up: frontend::build
	docker compose -f infra/docker-compose.test.yml --env-file frontend/.env.test up -d --build --wait

test-down:
	docker compose -f infra/docker-compose.test.yml --env-file frontend/.env.test down -v

test: test-up backend::test frontend::test test-down
