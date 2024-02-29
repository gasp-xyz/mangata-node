FROM debian:buster

ARG BINARY_PATH=target/release/rollup-node

WORKDIR /app

COPY /app/${BINARY_PATH} /app/

ENTRYPOINT ["/app/node"]
