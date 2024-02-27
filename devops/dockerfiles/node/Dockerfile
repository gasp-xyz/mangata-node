FROM debian:buster

ARG BUILD_DIR=target/release

WORKDIR /rollup

COPY $BUILD_DIR/rollup-node /rollup/node
COPY $BUILD_DIR/wbuild/rollup-runtime/rollup_runtime.compact.compressed.wasm /rollup/

ENTRYPOINT ["/rollup/node"]
