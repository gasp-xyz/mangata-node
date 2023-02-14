#!/bin/bash
REPO_ROOT=$(dirname $(readlink -f $0))/../

mkdir ./benchmarks

${REPO_ROOT}/target/release/mangata-node benchmark pallet \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet $1 \
    --extrinsic 'multiswap_sell_asset,multiswap_buy_asset' \
    --steps 20 \
    --repeat 5 \
    --output ./benchmarks/$1_weights.rs \
    --template ./templates/module-weight-template.hbs \
    &> ./benchmarks/benchmark_$1.txt
