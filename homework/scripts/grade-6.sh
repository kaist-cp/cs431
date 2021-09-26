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

# 1. Basic tests (30)
basic_faild=false
RUNNERS=(
    "cargo"
    "cargo --release"
)

echo "1. Running basic tests..."
for RUNNER in "${RUNNERS[@]}"; do
    if [ "$basic_faild" = false ]; then
        echo "Testing hazard_pointer.rs with $RUNNER..."
        TESTS=(
            "--test hazard_pointer -- --exact counter"
            "--test hazard_pointer -- --exact counter_sleep"
            "--test hazard_pointer -- --exact counter_tag"
            "--test hazard_pointer -- --exact stack"
            "--test hazard_pointer -- --exact two_stacks"
        )
        if [ $(run_tests) -ne 0 ]; then
            basic_faild=true
        fi
    fi
done

# 2. AddressSanitizer (40)
asan_failed=$basic_faild
RUNNERS=(
    "cargo_asan"
    "cargo_asan --release"
)
echo "2. Running AddressSanitizer tests..."
for RUNNER in "${RUNNERS[@]}"; do
    if [ "$basic_faild" = false ] && [ "$asan_failed" = false ]; then
        echo "Testing hazard_pointer.rs with $RUNNER..."
        TESTS=(
            "--test hazard_pointer -- --exact counter"
            "--test hazard_pointer -- --exact counter_sleep"
            "--test hazard_pointer -- --exact counter_tag"
            "--test hazard_pointer -- --exact stack"
            "--test hazard_pointer -- --exact two_stacks"
        )
        if [ $(run_tests) -ne 0 ]; then
            asan_failed=true
        fi
    fi
done

# 3. Synchronization (30)
sync_failed=$basic_faild
TIMEOUT=2m
RUNNER="cargo --features check-loom"
echo "3. Running synchronization tests..."
if [ "$basic_faild" = false ] && [ "$sync_failed" = false ]; then
    echo "Testing hazard_pointer.rs with $RUNNER..."
    TESTS=(
        "--test hazard_pointer sync::protect_collect_sync -- --nocapture"
        "--test hazard_pointer sync::get_protected_collect_sync -- --nocapture"
        "--test hazard_pointer sync::shield_drop_all_hazards_sync -- --nocapture"
    )
    if [ $(run_tests) -ne 0 ]; then
        sync_failed=true
    fi
fi

SCORE=0
if [ "$basic_faild" = false ]; then
    SCORE=$((SCORE + 30))
fi
if [ "$asan_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi
if [ "$sync_failed" = false ]; then
    SCORE=$((SCORE + 30))
fi
echo "Score: $SCORE / 100"
