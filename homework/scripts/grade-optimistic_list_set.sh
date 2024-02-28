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
optimistic_fine_grained_fail=${#COMMON_TESTS[@]}
others_failed=false

for r in "${!RUNNERS[@]}"; do
    RUNNER=${RUNNERS[r]}
    TIMEOUT=${RUNNER_TIMEOUTS[r]}
    for t in "${!COMMON_TESTS[@]}"; do
        TEST_NAME=${COMMON_TESTS[t]}
        # run only if no test has failed yet
        if [ $t -lt $optimistic_fine_grained_fail ]; then
            echo "Testing optimistic_fine_grained $TEST_NAME with $RUNNER, timeout $TIMEOUT..."
            TESTS=("--test list_set -- --exact optimistic_fine_grained::$TEST_NAME")
            for ((i = 0; i < REPS; i++)); do
                if [ $(run_tests) -ne 0 ]; then
                    optimistic_fine_grained_fail=$t
                    break
                fi
            done
        fi
    done

    if [ "$others_failed" == false ]; then
        echo "Running additional tests for optimistic_fine_grained with $RUNNER, timeout $TIMEOUT..."
        TESTS=(
            "--test list_set -- --exact optimistic_fine_grained::read_no_block"
            "--test list_set -- --exact optimistic_fine_grained::iter_invalidate_end"
            "--test list_set -- --exact optimistic_fine_grained::iter_invalidate_deleted"
        )
        if [ $(run_tests) -ne 0 ]; then
            others_failed=true
        fi
    fi
done

SCORES=( 0 5 15 30 45 )
SCORE=$(( SCORES[optimistic_fine_grained_fail] ))
if [ "$others_failed" == false ]; then
    SCORE=$(( SCORE + 10 ))
fi

echo "Score: $SCORE / 55"
