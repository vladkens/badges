tag=badges

dev:
	watchexec -r -- cargo run

lint:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo check --release --locked

update:
	git submodule update --init --recursive
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
