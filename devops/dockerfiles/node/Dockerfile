FROM debian:buster

ARG BINARY_PATH=target/release/rollup-node
ARG BINARY_NAME=rollup-node

WORKDIR /app

COPY $BUILD_DIR/${BINARY_NAME} /app/node

ENTRYPOINT ["/app/node"]
