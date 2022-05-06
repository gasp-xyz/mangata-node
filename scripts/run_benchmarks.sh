#!/bin/bash
REPO_ROOT=$(dirname $(readlink -f $0))/../

mkdir ./benchmarks

benchmarks=(
    "frame_system"
    "orml_tokens"
    "pallet_bootstrap"
    "pallet_collective"
    "pallet_elections_phragmen"
    "pallet_session"
    "pallet_timestamp"
    "pallet_treasury"
    "pallet_xyk"
    "parachain_staking"
    "xcm_asset_registry"
)

for bench in ${benchmarks[@]}; do
    ${REPO_ROOT}/scripts/run_benchmark.sh $bench
done
