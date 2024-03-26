#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

lines=$(grep_skip_comment "thread::spawn" "$BASEDIR/../src/boc.rs")
if [ -n "$lines" ]; then
    echo "thread::spawn(...) is not allowed."
    echo "$lines"
    exit 1
fi

RUNNERS=(
    "cargo"
    "cargo --release"
    "cargo_asan"
    "cargo_asan --release"
    "cargo_tsan"
    "cargo_tsan --release"
)
TIMEOUT=180s
SCORE=0

echo "1. Basic tests"

TESTS=(
    "--doc boc"
    "--test boc basic_test"
)

basic_test_failed=false

for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER, timeout $TIMEOUT..."
    if [ $(run_tests) -ne 0 ]; then
        basic_test_failed=true
        break
    fi
done

if [ "$basic_test_failed" = false ]; then
    SCORE=$((SCORE + 60))
fi

echo "2. Stress tests"

TESTS=(
    "--test boc stress_test"
)

stress_test_failed=false

for RUNNER in "${RUNNERS[@]}"; do
    echo "Running with $RUNNER, timeout $TIMEOUT..."
    if [ $(run_tests) -ne 0 ]; then
        stress_test_failed=true
        break
    fi
done

if [ "$stress_test_failed" = false ]; then
    SCORE=$((SCORE + 40))
fi

echo "Score: $SCORE / 100"
