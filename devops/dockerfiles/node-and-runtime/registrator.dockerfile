FROM debian:stretch
WORKDIR /code
RUN apt-get update -y && apt-get install -y curl
RUN curl -sL https://deb.nodesource.com/setup_16.x | bash -
RUN apt-get install -y nodejs
RUN apt-get install -y npm
COPY ./devops/dockerfiles/node-and-runtime/para-registrator/index.js /code
COPY ./devops/dockerfiles/node-and-runtime/para-registrator/package.json /code
RUN npm install
ENTRYPOINT []
