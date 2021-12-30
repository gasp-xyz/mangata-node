FROM parity/polkadot:v0.9.12 
USER root
RUN apt-get -y update && apt-get -y install curl jq
ENTRYPOINT ["/bin/bash", "-c"]
