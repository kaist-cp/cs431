#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

lines=$(grep_skip_comment transmute "$BASEDIR/../src/arc.rs")
if [ -n "$lines" ]; then
    echo "transmute() is not allowed."
    echo "$lines"
    exit 1
fi

SCORE=0

echo "1. Checking uses of SeqCst... skipped"
# # 1. SeqCst is not allowed.
# echo "1. Checking uses of SeqCst"
# lines=$(grep_skip_comment SeqCst "$BASEDIR/../src/arc.rs")
# if [ -n "$lines" ]; then
#     echo "You used SeqCst!"
#     echo "$lines"
#     echo "Score: 0 / 50"
#     exit
# fi

# 2. Basic arc functionality
echo "2. Running basic functionality tests"
RUNNERS=(
    "cargo"
    "cargo --release"
    "cargo_asan"
    "cargo_asan --release"
)
TESTS=(
    "--doc arc"
    "--test arc"
)
arc_basic_failed=false
for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER..."
    if [ $(run_tests) -ne 0 ]; then
        arc_basic_failed=true
        break
    fi
done

if [ "$arc_basic_failed" = false ]; then
    SCORE=$((SCORE + 25))
fi

# 3. Correctness
echo "3. Running correctness tests"
RUNNER="cargo --features check-loom"
TESTS=("--test arc")
echo "Running with $RUNNER..."
if [ $(run_tests) -eq 0 ]; then
    # SCORE=$((SCORE + 25))
    SCORE=$((SCORE + 15))
fi

# echo "Score: $SCORE / 50"
echo "Score: $SCORE / 40"
