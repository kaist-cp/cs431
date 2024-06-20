#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

RUNNERS=(
    "cargo"
    "cargo --release"
)


cache_basic_failed=false
cache_nonblocking_failed=false
tcp_failed=false
thread_pool_failed=false

# Executes test for each runner.
for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER..."

    if [ "$cache_basic_failed" = false ]; then
        echo "    Testing basic functionalities of cache.rs..."
        TESTS=(
            "--test cache -- --exact cache_no_duplicate_sequential"
            "--test cache -- --exact cache_no_duplicate_concurrent"
        )
        if [ $(run_tests) -ne 0 ]; then
            cache_basic_failed=true
        fi
    fi

    if [ "$cache_nonblocking_failed" = false ]; then
        echo "    Testing nonblockingness of cache.rs..."
        TESTS=(
            "--test cache -- --exact cache_no_block_disjoint"
            "--test cache -- --exact cache_no_reader_block"
        )
        if [ $(run_tests) -ne 0 ]; then
            cache_nonblocking_failed=true
        fi
    fi

    if [ "$tcp_failed" = false ]; then
        echo "    Testing tcp.rs..."
        TESTS=("--test tcp")
        if [ $(run_tests) -ne 0 ]; then
            tcp_failed=true
        fi
    fi

    if [ "$thread_pool_failed" = false ]; then
        echo "    Testing thread_pool.rs..."
        TESTS=("--test thread_pool")
        if [ $(run_tests) -ne 0 ]; then
            thread_pool_failed=true
        fi
    fi
done

SCORE=0

# Scores for cache.rs
if [ "$cache_basic_failed" = false ]; then
    SCORE=$((SCORE + 15))
fi
if [ "$cache_nonblocking_failed" = false ]; then
    SCORE=$((SCORE + 25))
fi

# Scores for tcp.rs
if [ "$tcp_failed" = false ]; then
    SCORE=$((SCORE + 20))
fi

# Scores for thread_pool.rs
if [ "$thread_pool_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi

echo "Score: $SCORE / 100"
