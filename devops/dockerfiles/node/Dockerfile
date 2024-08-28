FROM debian:buster

ARG BUILD_PATH=target/release

WORKDIR /app

COPY ${BUILD_PATH}/rollup-node /app/node
COPY ${BUILD_PATH}/wbuild/rollup-runtime/rollup_runtime.compact.compressed.wasm /app/

ENTRYPOINT ["/app/node"]
