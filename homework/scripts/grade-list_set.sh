#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

export RUST_TEST_THREADS=1


REPS=3
COMMON_TESTS=(
    "stress_sequential"
    "stress_concurrent"
    "log_concurrent"
    "iter_consistent"
)
RUNNERS=(
    "cargo --release"
    "cargo_asan"
    "cargo_asan --release"
    "cargo_tsan --release"
)
# timeout for each RUNNER
RUNNER_TIMEOUTS=(
    30s
    180s
    180s
    180s
)
# the index of the last failed test
fine_grained_fail=${#COMMON_TESTS[@]}

for r in "${!RUNNERS[@]}"; do
    RUNNER=${RUNNERS[r]}
    TIMEOUT=${RUNNER_TIMEOUTS[r]}
    for t in "${!COMMON_TESTS[@]}"; do
        TEST_NAME=${COMMON_TESTS[t]}
        # run only if no test has failed yet
        if [ $t -lt $fine_grained_fail ]; then
            echo "Testing fine_grained $TEST_NAME with $RUNNER, timeout $TIMEOUT..."
            TESTS=("--test list_set -- --exact fine_grained::$TEST_NAME")
            for ((i = 0; i < REPS; i++)); do
                if [ $(run_tests) -ne 0 ]; then
                    fine_grained_fail=$t
                    break
                fi
            done
        fi
    done
done

SCORES=( 0 5 15 30 45 )
SCORE=$(( SCORES[fine_grained_fail] ))

echo "Score: $SCORE / 45"
