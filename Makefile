.PHONY: help docker-up docker-down migrate-up migrate-down migrate-fresh migrate-status

help:
	@echo "Available commands:"
	@echo "  docker-up       - Start PostgreSQL and Redis containers"
	@echo "  docker-down     - Stop and remove containers"
	@echo "  migrate-up      - Run database migrations"
	@echo "  migrate-down    - Rollback last migration"
	@echo "  migrate-fresh   - Drop all tables and re-run migrations"
	@echo "  migrate-status  - Show migration status"

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

migrate-up:
	cd migrations && cargo run -- up

migrate-down:
	cd migrations && cargo run -- down

migrate-fresh:
	cd migrations && cargo run -- fresh

migrate-status:
	cd migrations && cargo run -- status

