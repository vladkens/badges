tag=badges

dev:
	watchexec -rc -e rs -- cargo run

fmt:
	cargo +nightly fmt
	cargo fix --allow-dirty --allow-staged

lint:
	cargo +nightly fmt --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cargo check --workspace --release --locked

test:
	cargo test --workspace --all-features --locked

update:
	git submodule update --init --recursive
	git submodule foreach 'git fetch --tags && git checkout $$(git describe --tags $$(git rev-list --tags --max-count=1))'
	rm -rf badgelib/src/_icons.rs badgelib/src/_width.rs
	cargo upgrade -i

deploy:
	fly deploy

docker-build:
	docker build -t $(tag) .
	docker images -q $(tag) | xargs docker inspect -f '{{.Size}}' | xargs numfmt --to=iec

docker-run: docker-build
	docker rm --force $(tag) || true
	docker run -p 8080:8080 --name $(tag) $(tag)

publish:
	cargo publish --manifest-path badgelib/Cargo.toml

bench:
	@# wrk -t4 -c400 -d30s http://localhost:8080/health
	wrk -t4 -c400 -d30s 'http://localhost:8080/badge/?icon=github&label=GitHub&value=badges'
