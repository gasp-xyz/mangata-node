FROM debian:buster

WORKDIR /mangata

COPY target/release/mangata-node /mangata/node
COPY target/release/wbuild/mangata-rococo-runtime/mangata_rococo_runtime.compact.compressed.wasm /mangata/
COPY target/release/wbuild/mangata-kusama-runtime/mangata_kusama_runtime.compact.compressed.wasm /mangata/

ENTRYPOINT ["/mangata/node"]