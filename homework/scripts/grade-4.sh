#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

TIMEOUT=1m
export RUST_TEST_THREADS=1

# 1. Basic (10)
RUNNERS=(
    "cargo"
    "cargo --release"
)
TESTS=(
    "--test list_set"
)

for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER..."
    if [ $(run_tests) -ne 0 ]; then
        echo "Score: 0 / 80"
        exit
    fi
done

# 2. Correctness (70)
RUNNERS=(
    "cargo_asan"
    "cargo_asan --release"
    "cargo_tsan"
    "cargo_tsan --release"
)
TESTS=(
    "--test list_set stress_sequential"
    "--test list_set stress_concurrent"
    "--test list_set log_concurrent"
    "--test list_set iter_consistent"
)

for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER..."
    if [ $(run_tests) -ne 0 ]; then
        echo "Score: 10 / 80"
        exit
    fi
done

echo "Score: 80 / 80"
