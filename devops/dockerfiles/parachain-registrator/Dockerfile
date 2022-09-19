FROM debian:stretch
WORKDIR /code
RUN apt-get update -y && apt-get install -y curl
RUN curl -sL https://deb.nodesource.com/setup_16.x | bash -
RUN apt-get install -y nodejs
RUN apt-get install -y npm vim
COPY ./devops/dockerfiles/parachain-registrator/src/index.js /code
COPY ./devops/dockerfiles/parachain-registrator/src/index_ksm.js /code
COPY ./devops/dockerfiles/parachain-registrator/src/package.json /code
COPY ./devops/parachain/config/genesis-state /code
COPY ./devops/parachain/config/genesis-wasm /code
RUN npm install
ENTRYPOINT []
