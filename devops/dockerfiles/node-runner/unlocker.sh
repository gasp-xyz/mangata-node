#!/bin/bash

echo "Unlocker: starting"

echo "Unlocker: dockerize blocks till 9933 ready for 30s"
dockerize -wait tcp://localhost:9933 -timeout 30s

echo "Unlocker: login to VAULT"

KUBE_TOKEN=$(cat /var/run/secrets/kubernetes.io/serviceaccount/token)

VAULT_TOKEN=$(curl --request POST \
       --data '{"jwt": "'"$KUBE_TOKEN"'", "role": "mangata"}' \
       $VAULT_ADDR/v1/auth/kubernetes/login | jq -r ".auth.client_token" )

vault login -no-print $VAULT_TOKEN

MAGIC=$(vault kv get -format=json ${SECRET_KEY} | jq ".data.data")

echo "Unlocker: importing secrets"

KEY_SEED="//xx"
ED_SEED=$(echo $MAGIC | jq -r ".ED_SEED")
ED_PUB_KEY=$(echo $MAGIC | jq -r ".ED_PUB_KEY")
RPC_ENDPOINT=$(echo $MAGIC | jq -r ".RPC_ENDPOINT")
KEY_TYPE=$(echo $MAGIC | jq -r ".KEY_TYPE")
￼


echo "Unlocker: Injecting keys"

curl -vH "Content-Type: application/json" \
        --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["'"${KEY_TYPE}"'", "'"${ED_SEED}"'", "'"${ED_PUB_KEY}"'"],"id":1 }' \
        "${RPC_ENDPOINT}"

