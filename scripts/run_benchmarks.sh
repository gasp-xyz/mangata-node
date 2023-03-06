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
    "orml_asset_registry"
    "pallet_treasury"
    "pallet_collective_mangata"
    "pallet_elections_phragmen"
    "pallet_crowdloan_rewards"
    "pallet_utility"
    "pallet_vesting_mangata"
    "pallet_issuance"
    "pallet_bootstrap"
    "pallet_multipurpose_liquidity"
    "pallet_sudo_origin"
    "pallet_token_timeout"
)

for bench in ${benchmarks[@]}; do
    ${REPO_ROOT}/scripts/run_benchmark.sh $bench
done
