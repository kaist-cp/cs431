#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

SCORE=0

# 1. Basic arc functionality
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

# 2. Correctness
RUNNER="cargo --features check-loom"
TESTS=("--test arc")
echo "Running with $RUNNER..."
if [ $(run_tests) -eq 0 ]; then
    SCORE=$((SCORE + 25))
fi

# 3. SeqCst is not allowed.
grep -n --color=always "SeqCst" $BASEDIR/../src/arc.rs
if [ $? -eq 0 ]; then
    echo "You used SeqCst!"
    SCORE=0
fi

echo "Score: $SCORE / 50"
