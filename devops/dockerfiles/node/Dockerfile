FROM liuchong/rustup:stable AS env_builder
LABEL description="Compiles all workspace artifacts"
ENV NODE_VERSION=16.13.0

WORKDIR /mangata
COPY . /mangata

ENV ETH_APP_ID=0xdd514baa317bf095ddba2c0a847765feb389c6a0
ENV ERC20_APP_ID=0x00e392c04743359e39f00cd268a5390d27ef6b44

RUN apt-get update && apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl llvm

RUN apt install -y curl 
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"
RUN node --version
RUN npm --version
RUN npm install --global yarn

RUN mkdir -p /mangata-e2e
RUN cd /mangata-e2e 
RUN git clone https://github.com/mangata-finance/mangata-e2e.git 
RUN cd mangata-e2e  
RUN cd mangata-e2e && git checkout feature/add-regular-checks 
RUN cd mangata-e2e && git pull && yarn install

RUN cd /mangata/

RUN rustup install nightly-2021-10-19
RUN rustup default nightly-2021-10-19
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-10-19
RUN cargo install cargo2junit

FROM env_builder as builder
RUN cargo build --release

FROM debian:stretch
WORKDIR /mangata
RUN apt-get update && apt-get install -y curl
COPY --from=builder /mangata/target/release/mangata-node /mangata/node
RUN /mangata/node --version
COPY devops/dockerfiles/node/runner.sh /runner.sh

ENTRYPOINT ["/mangata/node"]
