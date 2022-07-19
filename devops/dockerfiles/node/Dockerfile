FROM mangatasolutions/node-runner:0.2
ARG NODE_BINARY
ARG WASM
COPY $NODE_BINARY /mangata/node
COPY $WASM /mangata/
ENTRYPOINT ["/mangata/node"]
