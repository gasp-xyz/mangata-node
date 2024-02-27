FROM debian:buster

ARG BUILD_DIR=target/release
ARG BINARY_NAME=rollup-node
ARG WASM_PATH=wbuild/rollup-runtime/rollup_runtime.compact.compressed.wasm

WORKDIR /app

COPY $BUILD_DIR/${BINARY_NAME} /app/node
COPY $BUILD_DIR/${WASM_PATH}} /app/

ENTRYPOINT ["/app/node"]
