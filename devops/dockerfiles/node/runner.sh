#!/bin/bash
HEADER='Content-Type:application/json';
PAYLOAD='{"id":1,"jsonrpc":"2.0","method":"system_localPeerId"}';
ARGS=$@
MAX_RETRY_COUNT=${MAX_RETRY_COUNT:-10}
RETRY_PERIOD=${RETRY_PERIOD:-1}
#put binry directly to output args
UPDATED_CMD="$1"
ARGS=${@:2}

function wait_for_node {
    ADDRESS=$1
    local cnt=1;
    echo "Waiting for node $ADDRESS RPC service, MAX_RETRY_COUNT=$MAX_RETRY_COUNT, RETRY_PERIOD=$RETRY_PERIOD"
    while ! curl --silent --connect-timeout 0.3 -H "$HEADER" -d "$PAYLOAD" http://${ADDRESS} > /dev/null ; do
        echo "$ADDRESS RPC service not ready yet, retyring in ${RETRY_PERIOD}s"
        sh -c "sleep $RETRY_PERIOD"
        cnt=$[ $cnt + 1 ]
        if [ "$cnt" -gt "$MAX_RETRY_COUNT" ]; then
            echo "$ADDRESS RPC service still not ready,giving up"
            exit -1
        fi
    done;
    echo "$ADDRESS REQ: $PAYLOAD"
    RESP=$(curl --silent --connect-timeout 0.3 -H "$HEADER" -d "$PAYLOAD" http://${ADDRESS})
    echo "$ADDRESS RESP: $RESP"
}

function fetch_node_id {
    ADDRESS=$1
    curl --silent --connect-timeout 0.3 -H "$HEADER" -d "$PAYLOAD" http://${ADDRESS} | sed -r 's/.*result":"(([0-9]|[A-Z]|[a-z])+).*/\1/g' 
}


for i in $ARGS; do 
    ADDR=$(echo $i | grep p2p | sed 's@/.*/\(.*\)/tcp/[0-9]\+/p2p/AUTO@\1@g')
    PORT=$(echo $i | grep p2p | sed 's@/.*/.*/tcp/\([0-9]\+\)/p2p/AUTO@\1@g')
    PORT=$(echo $i | grep p2p | sed 's@/.*/.*/tcp/\([0-9]\+\)/p2p/AUTO@\1@g')
    if [ -n "$ADDR" ] && [ -n "$PORT" ];then
        # verbose logs
        wait_for_node $ADDR:$PORT
        # fetch id once its ready
        NODE_ID=$(fetch_node_id $ADDR:$PORT 2>/dev/null)
        UPDATED_ARG=$(echo $i | sed "s@AUTO@$NODE_ID@g")
        UPDATED_CMD="$UPDATED_CMD $UPDATED_ARG"
    else
        UPDATED_CMD="$UPDATED_CMD $i"
    fi
done

bash -xc "$UPDATED_CMD"
