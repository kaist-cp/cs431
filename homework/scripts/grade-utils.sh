#!/usr/bin/env bash

# Global variables
# * TEMPLATE_REV: git revision of the latest homework template
# * TESTS: array of test names
# * RUNNERS: array of "cargo[_asan | _tsan] [--release]"

echo_err() {
    echo -e "\033[0;31m\033[1m$@\033[0m" 1>&2
}
export -f echo_err

# check_diff FILE TEST_LINES_FROM_TAIL
# Abort if tests are modified.
# Uses global variable TEMPLATE_REV.
check_diff() {
    local FILE=$1
    local TAIL_N=$2
    diff <(tail -n $TAIL_N <(git show $TEMPLATE_REV:$FILE)) <(tail -n $TAIL_N $FILE) \
        || (echo_err "You modified tests for ${FILE}!" && exit 1)
}
export -f check_diff

# example: cargo_tsan test --release TEST_NAME 
cargo_asan() {
    local CMD=$1; shift
    RUSTFLAGS="-Z sanitizer=address" \
        cargo +nightly $CMD --target x86_64-unknown-linux-gnu $@ 
}
export -f cargo_asan

cargo_tsan() {
    local CMD=$1; shift
    RUSTFLAGS="-Z sanitizer=thread" \
        TSAN_OPTIONS="suppressions=suppress_tsan.txt" \
        RUST_TEST_THREADS=1 \
        cargo +nightly $CMD --target x86_64-unknown-linux-gnu $@ 
}
export -f cargo_tsan

# example: run_tests_with cargo_tsan --release
# echos number of failed tests
# Uses global variable TESTS
run_tests_with() {
    local CARGO=$1; shift
    $CARGO build $@ 2>/dev/null

    local FAILED=0
    for TEST in "${TESTS[@]}"; do
        local TEST_CMD="$CARGO test $TEST $@"
        timeout 10s bash -c "$TEST_CMD &>/dev/null"
        case $? in
            0) ;;
            124) echo_err "Test timed out: $TEST_CMD"; FAILED=$((FAILED + 1));;
            *)   echo_err "Test failed:    $TEST_CMD"; FAILED=$((FAILED + 1));;
        esac
    done
    echo $FAILED
}
export -f run_tests_with

# example: run_tests
# Uses global variable RUNNER
run_tests() {
    local FAILED=0
    # "cargo --relase" should be split into "cargo" and "--release"
    local IFS=' '
    echo $(run_tests_with $RUNNER)
}
