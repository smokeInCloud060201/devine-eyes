DOCKER_NETWORK_NAME=eye-devine-network
DOCKER_BIN=docker

.PHONY: help docker-up docker-down docker-clean build-api build-worker build-all create-network migrate-up migrate-down migrate-fresh migrate-status


help:
	@echo "Available commands:"
	@echo "  docker-up       - Start all containers (PostgreSQL, Redis, API, Worker)"
	@echo "  docker-down     - Stop and remove containers"
	@echo "  docker-clean    - Stop containers and remove volumes"
	@echo "  build-api       - Build the API Docker image"
	@echo "  build-worker    - Build the Worker Docker image"
	@echo "  build-all       - Build both API and Worker images"
	@echo "  create-network  - Create Docker network if it doesn't exist"
	@echo "  migrate-up      - Run database migrations"
	@echo "  migrate-down    - Rollback last migration"
	@echo "  migrate-fresh   - Drop all tables and re-run migrations"
	@echo "  migrate-status  - Show migration status"

docker-up: create-network
	@echo "Starting Docker containers..."
	@docker-compose -f ./deploy/docker-compose-local.yml down 2>/dev/null || true
	@docker-compose -f ./deploy/docker-compose-local.yml -p eyes-devine up -d
	@echo "Containers started successfully!"

docker-down:
	docker-compose -f ./deploy/docker-compose-local.yml down

docker-clean:
	docker-compose -f ./deploy/docker-compose-local.yml down -v
	docker rm -f docker-monitor-postgres docker-monitor-redis eyes-devine-server eyes-devine-worker 2>/dev/null || true

build-api:
	@echo "Building API Docker image..."
	docker build -f ./deploy/Dockerfile/api.Dockerfile -t eye-devine-api:latest .
	@echo "API image built successfully!"

build-worker:
	@echo "Building Worker Docker image..."
	docker build -f ./deploy/Dockerfile/worker.Dockerfile -t eye-devine-worker:latest .
	@echo "Worker image built successfully!"

build-all: build-api build-worker
	@echo "All images built successfully!"

migrate-up:
	cd migrations && cargo run -- up

migrate-down:
	cd migrations && cargo run -- down

migrate-fresh:
	cd migrations && cargo run -- fresh

migrate-status:
	cd migrations && cargo run -- status


create-network:
	@echo "Creating Docker network if it doesn't exist..."
	@$(DOCKER_BIN) network inspect $(DOCKER_NETWORK_NAME) >/dev/null 2>&1 || $(DOCKER_BIN) network create $(DOCKER_NETWORK_NAME)
	@echo "Network ready!"
