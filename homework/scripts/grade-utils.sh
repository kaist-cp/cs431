#!/usr/bin/env bash

# Global variables
# * TEMPLATE_REV: git revision of the latest homework template
# * TESTS: array of "[TARGET] [TEST_NAME] [-- <args>...]"
#   e.g. "--test linked_list", "--lib cache", "--test list_set -- --test-thread 1"
# * RUNNERS: array of "cargo[_asan | _tsan] [--release]"
# * TIMEOUT: default 10s

# rustup toolchain update stable nightly
# TODO: https://github.com/rust-lang/rust/issues/91689
export RUST_NIGHTLY=2021-12-05

echo_err() {
    echo -e "\033[0;31m\033[1m$@\033[0m" 1>&2
}
export -f echo_err

# check_diff FILE TEST_LINES_FROM_TAIL
# Abort if "--lib" tests are modified.
# Uses global variable TEMPLATE_REV.
check_diff() {
    local FILE=$1
    local TAIL_N=$2
    diff <(tail -n $TAIL_N <(git show $TEMPLATE_REV:$FILE)) <(tail -n $TAIL_N $FILE) \
        || (echo_err "You modified tests for ${FILE}!"; exit 1)
}
export -f check_diff

# Returns non-zero exit code if any of the linters have failed.
run_linters() {
    cargo fmt -- --check
    cargo clippy
    return 0
    # local FMT_ERR=$?
    # cargo clippy -- -D warnings
    # local CLIPPY_ERR=$?
    # [ "$FMT_ERR" -ne 0 ] && echo_err 'Please format your code with `cargo fmt` first.'
    # [ "$CLIPPY_ERR" -ne 0 ] && echo_err 'Please fix the issues from `cargo clippy` first.'
    # return $(( FMT_ERR || CLIPPY_ERR ))
}
export -f run_linters

# usage: cargo_asan [SUBCOMMAND] [OPTIONS] [-- <args>...]
# example: cargo_asan test --release TEST_NAME -- --skip SKIPPED
cargo_asan() {
    local SUBCOMMAND=$1; shift
    RUSTFLAGS="-Z sanitizer=address" \
        RUSTDOCFLAGS="-Z sanitizer=address" \
        cargo +nightly-$RUST_NIGHTLY $SUBCOMMAND --target x86_64-unknown-linux-gnu $@
}
export -f cargo_asan

cargo_tsan() {
    local SUBCOMMAND=$1; shift
    RUSTFLAGS="-Z sanitizer=thread" \
        TSAN_OPTIONS="suppressions=suppress_tsan.txt" \
        RUSTDOCFLAGS="-Z sanitizer=thread" \
        RUST_TEST_THREADS=1 \
        cargo +nightly-$RUST_NIGHTLY $SUBCOMMAND --target x86_64-unknown-linux-gnu $@
}
export -f cargo_tsan

# usage: _run_tests_with CARGO [OPTIONS]
# example: _run_tests_with cargo_tsan --release
# Echos number of failed tests to stdout.
# Echos error message to stderr.
# Uses global variable TESTS, TIMEOUT.
# [OPTIONS] must not contain " -- " (cargo options only).
_run_tests_with() {
    local CARGO=$1; shift
    local MSGS # https://mywiki.wooledge.org/BashPitfalls#local_var.3D.24.28cmd.29
    MSGS=$($CARGO test --no-run $@ 2>&1)
    if [ $? -ne 0 ]; then
        echo_err "Build failed! Error message:"
        echo "${MSGS}" 1>&2
        echo_err "--------------------------------------------------------------------------------"
        echo ${#TESTS[@]} # failed all tests
        exit 1
    fi

    local FAILED=0
    for TEST in "${TESTS[@]}"; do
        local TEST_CMD="$CARGO test $@ $TEST"
        timeout ${TIMEOUT:-10s} bash -c "$TEST_CMD 2>/dev/null" 1>&2
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
