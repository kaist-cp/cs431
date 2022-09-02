#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

export RUST_TEST_THREADS=1

TEST_NAMES=(
    "stress_sequential"
    "lookup_concurrent"
    "insert_concurrent"
    "stress_concurrent"
    "log_concurrent"
)
RUNNERS=(
    "cargo"
    "cargo --release"
    "cargo_asan"
    "cargo_asan --release"
    "cargo_tsan --release"
)
# timeout for each (TEST_NAME, RUNNER).
TIMEOUTS=(
    10s 10s 10s  10s 10s
    10s 10s 10s  10s 10s
    10s 10s 10s  10s 10s
    30s 10s 120s 15s 60s
    30s 10s 120s 15s 60s
)
# the index of the last failed test
growable_array_fail=${#TEST_NAMES[@]}
split_ordered_list_fail=${#TEST_NAMES[@]}

for t in "${!TEST_NAMES[@]}"; do
    for r in "${!RUNNERS[@]}"; do
        TEST_NAME=${TEST_NAMES[t]}
        RUNNER=${RUNNERS[r]}
        TIMEOUT=${TIMEOUTS[ t * ${#RUNNERS[@]} + r ]}
        # run only if no test has failed yet
        if [ $t -lt $growable_array_fail ]; then
            echo "Testing growable_array $TEST_NAME with $RUNNER, timeout $TIMEOUT..."
            TESTS=("--test growable_array $TEST_NAME")
            if [ $(run_tests) -ne 0 ]; then
                growable_array_fail=$t
            fi
        fi
        if [ $t -lt $split_ordered_list_fail ]; then
            echo "Testing split_ordered_list $TEST_NAME with $RUNNER, timeout $TIMEOUT..."
            TESTS=("--test split_ordered_list $TEST_NAME")
            if [ $(run_tests) -ne 0 ]; then
                split_ordered_list_fail=$t
            fi
        fi
    done
done

SCORES=( 0 10 20 30 60 90 )
SCORE=$(( ${SCORES[growable_array_fail]} + ${SCORES[split_ordered_list_fail]} ))
echo "Score: $SCORE / 180"
