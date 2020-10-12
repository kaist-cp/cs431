#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

# Checks if the submission didn't alter the test cases.
TEMPLATE_REV=HEAD
check_diff ./src/hello_server/cache.rs 79
check_diff ./src/hello_server/tcp.rs 41
check_diff ./src/hello_server/thread_pool.rs 72

RUNNERS=(
    "cargo"
    "cargo --release"
    "cargo_asan"
    "cargo_asan --release"
    "cargo_tsan"
    "cargo_tsan --release"
)


t1_failed=false
t2_failed=false
t3_failed=false

# Executes test for each runner.
for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER..."

    if [ "$t1_failed" = false ]; then
        echo "    Testing cache.rs..."
        TESTS=("--lib hello_server::cache")
        if [ $(run_tests) -ne 0 ]; then
            t1_failed=true
        fi
    fi

    if [ "$t2_failed" = false ]; then
        echo "    Testing tcp.rs..."
        TESTS=("--lib hello_server::tcp")
        if [ $(run_tests) -ne 0 ]; then
            t2_failed=true
        fi
    fi

    if [ "$t3_failed" = false ]; then
        echo "    Testing thread_pool.rs..."
        TESTS=("--lib hello_server::thread_pool")
        if [ $(run_tests) -ne 0 ]; then
            t3_failed=true
        fi
    fi
done

SCORE=0

# Scores for cache.rs
if [ "$t1_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi

# Scores for tcp.rs
if [ "$t2_failed" = false ]; then
    SCORE=$((SCORE + 20))
fi

# Scores for thread_pool.rs
if [ "$t3_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi

echo "Score: $SCORE / 100"
