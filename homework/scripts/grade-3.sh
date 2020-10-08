#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

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
# Executes test for each runner.
for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER..."
    if [ $(run_tests) -ne 0 ]; then
        arc_basic_failed=true
        break
    fi
done

SCORE=0

# Scores for basic arc functionality
if [ "$arc_basic_failed" = false ]; then
    SCORE=$((SCORE + 25))
fi

grep -n --color=always "SeqCst" $BASEDIR/../src/arc.rs
if [ $? -eq 0 ]; then
    echo "You used SeqCst!"
    SCORE=0
fi

echo "Score: $SCORE / 50"
