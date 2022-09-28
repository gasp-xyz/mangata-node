build:
	docker buildx build -f devops/dockerfiles/node-new/Dockerfile \
		-t mangatasolutions/mangata-node:devm \
		--cache-from=type=registry,ref=mangatasolutions/mangata-node \
		--cache-to=type=registry,ref=mangatasolutions/mangata-node,mode=max \
		--push --builder=container .

build2:
	docker volume inspect cargo-cache > /dev/null || docker volume create cargo-cache
	docker run --rm -it --platform linux/amd64 -w /code \
		-v $(CURDIR):/code \
		-v cargo-cache:'/cache/' \
		-e CARGO_HOME=/cache/cargo/ \
		-e SCCACHE_DIR=/cache/sccache/ \
		-e CARGO_TARGET_DIR="/code/docker-cargo2" \
		paritytech/ci-linux:production \
		cargo build --manifest-path=/code/Cargo.toml --release --features=mangata-rococo