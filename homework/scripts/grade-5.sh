#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

TIMEOUT=3m
export RUST_TEST_THREADS=1


# 1. Basic tests (30 each)
growable_array_basic_failed=false
split_ordered_list_basic_failed=false
RUNNERS=(
    "cargo"
    "cargo --release"
)
echo "1. Running basic tests..."
for RUNNER in "${RUNNERS[@]}"; do
    if [ "$growable_array_basic_failed" = false ]; then
        echo "Testing growable_array.rs with $RUNNER..."
        TESTS=(
            "--test growable_array smoke"
            "--test growable_array stress_sequential"
            "--test growable_array stress_concurrent"
            "--test growable_array log_concurrent"
        )
        if [ $(run_tests) -ne 0 ]; then
            growable_array_basic_failed=true
        fi
    fi

    if [ "$split_ordered_list_basic_failed" = false ]; then
        echo "Testing split_ordered_list.rs with $RUNNER..."
        TESTS=(
            "--test split_ordered_list smoke"
            "--test split_ordered_list stress_sequential"
            "--test split_ordered_list stress_concurrent"
            "--test split_ordered_list log_concurrent"
        )
        if [ $(run_tests) -ne 0 ]; then
            split_ordered_list_basic_failed=true
        fi
    fi
done

# 2. AddressSanitizer (40 each)
growable_array_asan_failed=$growable_array_basic_failed
split_ordered_list_asan_failed=$split_ordered_list_basic_failed
RUNNERS=(
    "cargo_asan"
    "cargo_asan --release"
)
echo "2. Running AddressSanitizer tests..."
for RUNNER in "${RUNNERS[@]}"; do
    if [ "$growable_array_basic_failed" = false ] && [ "$growable_array_asan_failed" = false ]; then
        echo "Testing growable_array.rs with $RUNNER..."
        TESTS=(
            "--test growable_array smoke"
            "--test growable_array stress_sequential"
            "--test growable_array stress_concurrent"
            "--test growable_array log_concurrent"
        )
        if [ $(run_tests) -ne 0 ]; then
            growable_array_asan_failed=true
        fi
    fi

    if [ "$split_ordered_list_basic_failed" = false ] && [ "$split_ordered_list_asan_failed" = false ]; then
        echo "Testing split_ordered_list.rs with $RUNNER..."
        TESTS=(
            "--test split_ordered_list smoke"
            "--test split_ordered_list stress_sequential"
            "--test split_ordered_list stress_concurrent"
            "--test split_ordered_list log_concurrent"
        )
        if [ $(run_tests) -ne 0 ]; then
            split_ordered_list_asan_failed=true
        fi
    fi
done

# 3. ThreadSanitizer (20 each)
growable_array_tsan_failed=$growable_array_basic_failed
split_ordered_list_tsan_failed=$split_ordered_list_basic_failed
# too slow without optimization
RUNNERS=("cargo_tsan --release")
echo "3. Running ThreadSanitizer tests..."
for RUNNER in "${RUNNERS[@]}"; do
    if [ "$growable_array_basic_failed" = false ] && [ "$growable_array_tsan_failed" = false ]; then
        echo "Testing growable_array.rs with $RUNNER..."
        TESTS=(
            "--test growable_array stress_concurrent"
            "--test growable_array log_concurrent"
        )
        if [ $(run_tests) -ne 0 ]; then
            growable_array_tsan_failed=true
        fi
    fi

    if [ "$split_ordered_list_basic_failed" = false ] && [ "$split_ordered_list_tsan_failed" = false ]; then
        echo "Testing split_ordered_list.rs with $RUNNER..."
        TESTS=(
            "--test split_ordered_list stress_concurrent"
            "--test split_ordered_list log_concurrent"
        )
        if [ $(run_tests) -ne 0 ]; then
            split_ordered_list_tsan_failed=true
        fi
    fi
done

SCORE=0
if [ "$growable_array_basic_failed" = false ]; then
    SCORE=$((SCORE + 30))
fi
if [ "$split_ordered_list_basic_failed" = false ]; then
    SCORE=$((SCORE + 30))
fi
if [ "$growable_array_asan_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi
if [ "$split_ordered_list_asan_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi
if [ "$growable_array_tsan_failed" = false ]; then
    SCORE=$((SCORE + 20))
fi
if [ "$split_ordered_list_tsan_failed" = false ]; then
    SCORE=$((SCORE + 20))
fi
echo "Score: $SCORE / 180"
