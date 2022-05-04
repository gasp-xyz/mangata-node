FROM debian:buster
COPY --from=mangatasolutions/node-builder:1.0 /tmp/profiler/target/release/bytehound /usr/bin/bytehound
COPY --from=mangatasolutions/node-builder:1.0 /tmp/profiler/target/release/libbytehound.so /usr/lib/libbytehound.so
ENV DOCKERIZE_VERSION v0.6.1
ENV VAULT_VERSION 1.9.3
RUN apt-get update && apt-get install -y curl netcat jq unzip
RUN curl -L https://github.com/jwilder/dockerize/releases/download/${DOCKERIZE_VERSION}/dockerize-alpine-linux-amd64-${DOCKERIZE_VERSION}.tar.gz --output /tmp/dockerize.tar.gz
RUN curl -L https://releases.hashicorp.com/vault/${VAULT_VERSION}/vault_${VAULT_VERSION}_linux_amd64.zip --output /tmp/vault.zip
RUN tar -C /usr/bin -xzvf /tmp/dockerize.tar.gz
RUN unzip /tmp/vault.zip -d /usr/bin
COPY devops/dockerfiles/node-runner/unlocker.sh /unlocker.sh
COPY devops/dockerfiles/node-runner/entrypoint.sh /entrypoint.sh
RUN mkdir /mangata
