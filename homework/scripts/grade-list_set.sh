#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

TIMEOUT=1m
export RUST_TEST_THREADS=1


# 1. Common tests (45 + 45)
echo "1. Running common tests"
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
# the index of the last failed test
fine_grained_fail=${#COMMON_TESTS[@]}
optimistic_fine_grained_fail=${#COMMON_TESTS[@]}

for r in "${!RUNNERS[@]}"; do
    for t in "${!COMMON_TESTS[@]}"; do
        TEST_NAME=${COMMON_TESTS[t]}
        RUNNER=${RUNNERS[r]}
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
done

SCORES=( 0 5 15 30 45 )
SCORE=$(( SCORES[fine_grained_fail] + SCORES[optimistic_fine_grained_fail] ))

# 2. other tests (5 + 5)
echo "2. Running other tests for optimistic_fine_grained"
RUNNER="cargo"
OTHER_TESTS=(
    "--test list_set -- --exact optimistic_fine_grained::read_no_block"
    "--test list_set -- --exact optimistic_fine_grained::iter_invalidate_end"
)
for TEST in "${OTHER_TESTS[@]}"; do
    echo "Running with $RUNNER..."
    TESTS=("$TEST")
    if [ $(run_tests) -eq 0 ]; then
        SCORE=$(( SCORE + 5 ))
    fi
done

echo "Score: $SCORE / 100"
