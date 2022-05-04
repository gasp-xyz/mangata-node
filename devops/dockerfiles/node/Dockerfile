FROM mangatasolutions/node-runner:1.0
ARG NODE_BINARY
ARG WASM
COPY $NODE_BINARY /mangata/node
COPY $WASM /mangata/
ENTRYPOINT /entrypoint.sh
