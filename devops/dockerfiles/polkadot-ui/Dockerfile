from ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive 
RUN apt-get -y update && apt-get -y install npm curl git make
RUN curl -sL https://deb.nodesource.com/setup_16.x -o /tmp/nodesource_setup.sh
RUN /bin/bash /tmp/nodesource_setup.sh
RUN apt-get -y update && apt-get -y  install nodejs
RUN npm install -g yarn
RUN mkdir /code
ENV HOME=/tmp
RUN git clone https://github.com/polkadot-js/apps -b v0.96.1 /tmp/apps
WORKDIR /tmp/apps
RUN yarn install
ENTRYPOINT yarn run start
