#!/bin/bash
REPO_ROOT=$(dirname $(readlink -f $0))/../

mkdir ./benchmarks

benchmarks=(
    "frame_system"
    "pallet_session"
    "pallet_timestamp"
    "orml_tokens"
    "parachain_staking"
    "pallet_xyk"
    "xcm_asset_registry"
    "pallet_treasury"
    "pallet_collective"
    "pallet_elections_phragmen"
)

for bench in ${benchmarks[@]}; do
    ${REPO_ROOT}/scripts/run_benchmark.sh $bench
done
