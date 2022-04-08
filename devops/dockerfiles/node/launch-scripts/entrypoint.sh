#!/bin/bash

echo "Running wrapper script"

if [[ ! -z ${VAULT_ADDR+z} ]]; then
    echo "Node: Vault set, running unlocker.sh"
    /etc/mangata/unlocker.sh &
else
    echo "Node: Vault is not set, skipping unlocker.sh"
fi

/mangata/node $@
