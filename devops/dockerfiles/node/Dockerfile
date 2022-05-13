FROM mangatasolutions/node-runner:0.1
ARG NODE_BINARY
ARG WASM
RUN mkdir /xxx
COPY . /xxx
RUN ls -la /xxx
COPY $NODE_BINARY /mangata/node
COPY $WASM /mangata/
ENTRYPOINT /entrypoint.sh
