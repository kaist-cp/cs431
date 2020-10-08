#!/usr/bin/env bash

# Global variables
# * TEMPLATE_REV: git revision of the latest homework template
# * TESTS: array of "[TARGET] [TEST_NAME]"
#   e.g. "--test linked_list", "--lib cache"
# * RUNNERS: array of "cargo[_asan | _tsan] [--release]"

rustup toolchain update stable nightly

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
        || (echo_err "You modified tests for ${FILE}!"; exit 1)
}
export -f check_diff

# usage: cargo_asan [SUBCOMMAND] [OPTIONS] [-- <args>...]
# example: cargo_asan test --release TEST_NAME -- --skip SKIPPED
cargo_asan() {
    local SUBCOMMAND=$1; shift
    RUSTFLAGS="-Z sanitizer=address" \
        RUSTDOCFLAGS="-Z sanitizer=address" \
        cargo +nightly $SUBCOMMAND --target x86_64-unknown-linux-gnu $@
}
export -f cargo_asan

cargo_tsan() {
    local SUBCOMMAND=$1; shift
    RUSTFLAGS="-Z sanitizer=thread" \
        TSAN_OPTIONS="suppressions=suppress_tsan.txt" \
        RUSTDOCFLAGS="-Z sanitizer=thread" \
        RUST_TEST_THREADS=1 \
        cargo +nightly $SUBCOMMAND --target x86_64-unknown-linux-gnu $@
}
export -f cargo_tsan

# usage: _run_tests_with CARGO [OPTIONS]
# example: _run_tests_with cargo_tsan --release
# echos number of failed tests
# Uses global variable TESTS
_run_tests_with() {
    local CARGO=$1; shift
    $CARGO test --no-run $@ &>/dev/null \
        || (echo_err "Build failed!"; exit 1)

    local FAILED=0
    for TEST in "${TESTS[@]}"; do
        local TEST_CMD="$CARGO test $TEST $@"
        timeout 10s bash -c "$TEST_CMD 2>/dev/null" 1>&2
        case $? in
            0) ;;
            124) echo_err "Test timed out: $TEST_CMD"; FAILED=$((FAILED + 1));;
            *)   echo_err "Test failed:    $TEST_CMD"; FAILED=$((FAILED + 1));;
        esac
    done
    echo $FAILED
}

# example: run_tests
# Uses global variable RUNNER and TESTS
run_tests() {
    # "cargo --relase" should be split into "cargo" and "--release"
    local IFS=' '
    echo $(_run_tests_with $RUNNER)
}
export -f run_tests
