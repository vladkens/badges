.PHONY: prepare check test watch build update deploy docker-build docker-run bench

tag=badges

prepare:
	cargo fmt
	cargo clippy --fix --all-targets --all-features --locked --allow-dirty --allow-staged -- -D warnings
	cargo check --release --locked

check:
	cargo fmt --check
	cargo clippy --all-targets --all-features --locked -- -D warnings
	cargo check --release --locked

test:
	cargo test --all-features --locked

watch:
	watchexec -rc -e rs -- cargo run

build:
	cargo build --release --locked
	ls -lh target/release/badges

update:
	cargo upgrade -i

deploy:
	fly deploy

docker-build:
	docker build -t $(tag) .
	docker images -q $(tag) | xargs docker inspect -f '{{.Size}}' | xargs numfmt --to=iec

docker-run: docker-build
	docker rm --force $(tag) || true
	docker run -p 8080:8080 --name $(tag) $(tag)

bench:
	@# wrk -t4 -c400 -d30s http://localhost:8080/health
	wrk -t4 -c400 -d30s 'http://localhost:8080/badge/?icon=github&label=GitHub&value=badges'
