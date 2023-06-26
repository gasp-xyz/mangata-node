FROM debian:buster

ARG BUILD_DIR=target/release

WORKDIR /mangata

COPY $BUILD_DIR/mangata-node /mangata/node
COPY $BUILD_DIR/wbuild/mangata-*-runtime/mangata_*_runtime.compact.compressed.wasm /mangata/
# COPY $BUILD_DIR/wbuild/mangata-kusama-runtime/mangata_kusama_runtime.compact.compressed.wasm /mangata/

ENTRYPOINT ["/mangata/node"]
