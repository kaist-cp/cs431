#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

export RUST_TEST_THREADS=1

TEMPLATE_REV=HEAD
check_diff ./src/hazard_pointer/hazard.rs 89
check_diff ./src/hazard_pointer/retire.rs 27

lines=$(grep_skip_comment transmute "$BASEDIR"/../src/hazard_pointer/{retire,hazard}.rs)
if [ -n "$lines" ]; then
    echo "transmute() is not allowed."
    echo "$lines"
    exit 1
fi

echo "1. Checking uses of SeqCst... skipped"
# # 1. Check uses of SeqCst
# performance_failed=false

# echo "1. Checking uses of SeqCst..."
# lines=$(grep_skip_comment SeqCst "$BASEDIR"/../src/hazard_pointer/{retire,hazard}.rs)
# if [ "$(echo -n "$lines" | grep -c '^')" -gt 2 ]; then
#     echo "You used SeqCst more than 2 times!"
#     echo "$lines"
#     performance_failed=true
# fi


echo "2. Running basic tests..."
RUNNERS=(
    "cargo"
    "cargo --release"
    "cargo_asan"
    "cargo_asan --release"
)
# # Use tsan for non-optimal solution.
# # In this case, we expect SeqCst for all accesses, which tsan understands.
# if [ "$performance_failed" = true ]; then
#     RUNNERS+=("cargo_tsan")
# fi
RUNNERS+=("cargo_tsan")
hazard_failed=false
retire_failed=false
integration_failed=false

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
            "--test hazard_pointer -- --exact queue"
            "--test hazard_pointer -- --exact stack_queue"
        )
        REPS=5
        for ((i = 0; i < REPS; i++)); do
            if [ $(run_tests) -ne 0 ]; then
                integration_failed=true
                break
            fi
        done
    fi
done


# # 3. Check relaxed memory synchronization
# # NOTE: We only accept optimal and correct solution.
# # So, if SeqCst > 2, no need to run check-loom test.
# # - This prevents running check-loom on the SC version
# #   (to avoid confusion caused by loom's inability to handle SeqCst accesses.)
# # - This assumes that there is no solution with SeqCst accesses ≤ 2.
# loom_failed=$performance_failed
# if [ "$performance_failed" = false ]; then
#     echo "3. Running synchronization tests..."
#     RUNNER="cargo --features check-loom"
#     TIMEOUT=2m
#     TESTS=(
#         "--test hazard_pointer sync::try_protect_collect_sync -- --nocapture"
#         "--test hazard_pointer sync::protect_collect_sync -- --nocapture"
#         "--test hazard_pointer sync::shield_drop_all_hazards_sync -- --nocapture"
#     )
#     if [ $(run_tests) -ne 0 ]; then
#         loom_failed=true
#     fi
# fi


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
# if [ "$loom_failed" = false ]; then
#     SCORE=$((SCORE + 30))
# fi
# echo "Score: $SCORE / 100"
echo "Score: $SCORE / 70"
