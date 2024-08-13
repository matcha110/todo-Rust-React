build:
	docker-compose build

db:
	docker-compose up -d

dev:
	sqlx db create
	sqlx migrate run
	cargo watch -x run

test:
	cargo test

test-s:
	cargo test --no-default-features

exec:
	docker-compose exec database bash