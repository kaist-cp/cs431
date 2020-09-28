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
    "--doc linked_list"
    "--test linked_list"
)


linked_list_failed=false
# Executes test for each runner.
for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER..."
    if [ $(run_tests) -ne 0 ]; then
        linked_list_failed=true
        break
    fi
done

SCORE=0

# Scores for linked_list.rs
if [ "$linked_list_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi

echo "Score: $SCORE / 40"
