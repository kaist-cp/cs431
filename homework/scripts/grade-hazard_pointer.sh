#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

export RUST_TEST_THREADS=1


# 1. Basic tests (20 + 10 + 40)
RUNNERS=(
    "cargo"
    "cargo --release"
    "cargo_asan"
    "cargo_asan --release"
)
hazard_failed=false
retire_failed=false
integration_failed=false

echo "1. Running basic tests..."
for RUNNER in "${RUNNERS[@]}"; do
    if [ "$hazard_failed" = false ]; then
        echo "Running basic tests in hazard.rs with $RUNNER..."
        TESTS=(
            "--lib -- --exact hazard_pointer::hazard::tests::all_hazards_protected"
            "--lib -- --exact hazard_pointer::hazard::tests::all_hazards_unprotected"
            "--lib -- --exact hazard_pointer::hazard::tests::recycle_slots"
        )
        if [ $(run_tests) -ne 0 ]; then
            hazard_failed=true
        fi
    fi

    if [ "$retire_failed" = false ]; then
        echo "Running a test in retire.rs with $RUNNER..."
        TESTS=(
            "--lib -- --exact hazard_pointer::retire::tests::retire_threshold_collect"
        )
        if [ $(run_tests) -ne 0 ]; then
            retire_failed=true
        fi
    fi

    if [ "$integration_failed" = false ]; then
        echo "Running tests in tests/hazard_pointer.rs with $RUNNER..."
        TESTS=(
            "--test hazard_pointer -- --exact counter"
            "--test hazard_pointer -- --exact counter_sleep"
            "--test hazard_pointer -- --exact stack"
            "--test hazard_pointer -- --exact two_stacks"
        )
        if [ $(run_tests) -ne 0 ]; then
            integration_failed=true
        fi
    fi
done

# 2. Synchronization (30)
RUNNER="cargo --features check-loom"
TIMEOUT=2m
loom_failed=false

echo "2. Running synchronization tests..."
TESTS=(
    "--test hazard_pointer sync::try_protect_collect_sync -- --nocapture"
    "--test hazard_pointer sync::protect_collect_sync -- --nocapture"
    "--test hazard_pointer sync::shield_drop_all_hazards_sync -- --nocapture"
)
if [ $(run_tests) -ne 0 ]; then
    loom_failed=true
fi

SCORE=0
if [ "$hazard_failed" = false ]; then
    SCORE=$((SCORE + 20))
fi
if [ "$retire_failed" = false ]; then
    SCORE=$((SCORE + 10))
fi
if [ "$integration_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi
if [ "$loom_failed" = false ]; then
    SCORE=$((SCORE + 30))
fi
echo "Score: $SCORE / 100"
