.PHONY: prepare check test build update clean

prepare:
	cargo +nightly fmt
	cargo clippy --fix --all-targets --all-features --locked --allow-dirty --allow-staged -- -D warnings
	cargo check --release --locked

check:
	cargo +nightly fmt --check
	cargo clippy --all-targets --all-features --locked -- -D warnings
	cargo check --release --locked

test:
	cargo test --all-features --locked

build:
	cargo build --release --locked
	ls -lh target/release/badges

update:
	cargo upgrade --incompatible
	cargo update

clean:
	cargo clean

# badges
.PHONY: watch smoke deploy docker-build docker-run bench

tag=badges

watch: # run the server, restarting on .rs changes
	watchexec -rc -e rs -- cargo run

smoke: # check that prod badges still show real upstream values (BASE_URL to override)
	uv run scripts/smoke.py

deploy: # deploy to fly.io
	fly deploy

docker-build: # build the docker image and print its size
	docker build -t $(tag) .
	docker images -q $(tag) | xargs docker inspect -f '{{.Size}}' | xargs numfmt --to=iec

docker-run: docker-build # run the docker image on port 8080
	docker rm --force $(tag) || true
	docker run -p 8080:8080 --name $(tag) $(tag)

bench: # load-test a local server with wrk
	@# wrk -t4 -c400 -d30s http://localhost:8080/health
	wrk -t4 -c400 -d30s 'http://localhost:8080/badge/?icon=github&label=GitHub&value=badges'
