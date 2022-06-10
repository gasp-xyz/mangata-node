FROM mangatasolutions/node-builder:0.1 as bytehound
RUN git clone --depth 1 --branch 0.8.0 https://github.com/koute/memory-profiler /tmp/profiler
RUN cargo build --manifest-path /tmp/profiler/Cargo.toml --release -p bytehound-preload -p bytehound-cli

FROM debian:buster
COPY --from=bytehound /tmp/profiler/target/release/bytehound /usr/bin/bytehound
COPY --from=bytehound /tmp/profiler/target/release/libbytehound.so /usr/lib/libbytehound.so
ENV DOCKERIZE_VERSION v0.6.1
ENV VAULT_VERSION 1.9.3
RUN apt-get update \
	&& apt-get install -y \
	curl \
	netcat \
	jq \
	bsdtar \
	&& rm -rf /var/lib/apt/lists/*
RUN curl -L https://github.com/jwilder/dockerize/releases/download/${DOCKERIZE_VERSION}/dockerize-alpine-linux-amd64-${DOCKERIZE_VERSION}.tar.gz | tar -xz -C /usr/bin
RUN curl -L https://releases.hashicorp.com/vault/${VAULT_VERSION}/vault_${VAULT_VERSION}_linux_amd64.zip | bsdtar -xvf - -C /usr/bin/
	
COPY devops/dockerfiles/node-runner/unlocker.sh /unlocker.sh
COPY devops/dockerfiles/node-runner/entrypoint.sh /entrypoint.sh
RUN mkdir /mangata
