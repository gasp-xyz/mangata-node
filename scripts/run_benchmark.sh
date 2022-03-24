#!/bin/bash
REPO_ROOT=$(dirname $(readlink -f $0))/../

mkdir ./benchmarks

${REPO_ROOT}/target/release/mangata-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet $1 \
    --extrinsic '*' \
    --steps 20 \
    --repeat 10 \
    --output ./benchmarks/$1_weights.rs \
    --template ./templates/module-weight-template.hbs \
    &> ./benchmarks/benchmark_$1.txt
