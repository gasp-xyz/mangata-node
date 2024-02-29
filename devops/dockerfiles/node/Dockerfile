FROM debian:buster

ARG BINARY_PATH=target/release/rollup-node

WORKDIR /app

COPY ${BINARY_PATH} /app/

ENTRYPOINT ["/app/node"]
