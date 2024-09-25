#!/bin/bash
REPO_ROOT=$(dirname $(readlink -f $0))/../

mkdir ./benchmarks
cd ./benchmarks

${REPO_ROOT}/target/release/rollup-node benchmark overhead \
    --chain rollup-local \
    -lblock_builder=debug \
    --max-ext-per-block 50000 \
    --base-path . \
    &>./overhead_benchmark.txt
