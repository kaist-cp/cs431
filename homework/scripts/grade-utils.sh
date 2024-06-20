#!/usr/bin/env bash

# Global variables
# * TEMPLATE_REV: git revision of the latest homework template
# * TESTS: array of "[TARGET] [TEST_NAME] [-- <args>...]"
#   e.g. "--test linked_list", "--lib cache", "--test list_set -- --test-thread 1"
# * RUNNERS: array of "cargo[_asan | _tsan] [--release]"
# * TIMEOUT: default 10s

export RUST_NIGHTLY=nightly
rustup toolchain update stable nightly
# rustup install $RUST_NIGHTLY
# rustup component add rust-src --toolchain $RUST_NIGHTLY-x86_64-unknown-linux-gnu


echo_err() {
    echo "$@" 1>&2
}
export -f echo_err

# check_diff FILE TEST_LINES_FROM_TAIL
# Abort if "--lib" tests are modified.
# Uses global variable TEMPLATE_REV.
check_diff() {
    local FILE=$1
    local TAIL_N=$2
    if ! diff -u <(tail -n "$TAIL_N" <(git show "$TEMPLATE_REV:$FILE")) <(tail -n "$TAIL_N" "$FILE"); then
        echo_err "You modified tests for ${FILE}!"
        exit 1
    fi
}
export -f check_diff

# grep_skip_comment PATTERN FILE...
# Shows all occurrences of PATTERN in code, excluding line comment.
grep_skip_comment() {
    local pat=$1; shift
    for file; do
        for linenr in $(sed 's://.*::' "$file" | grep -n "$pat" | cut -d : -f 1); do
            sed -n "${linenr}p" "$file" | awk -v F="${file##*/}" -v L="$linenr" '{print F ":" L ":" $0}'
        done
    done
}
export -f grep_skip_comment

# Returns non-zero exit code if any of the linters have failed.
run_linters() {
    cargo fmt -- --check
    local FMT_ERR=$?
    cargo +$RUST_NIGHTLY clippy -- -D warnings
    local CLIPPY_ERR=$?
    [ "$FMT_ERR" -ne 0 ] && echo_err 'Please format your code with `cargo fmt` first.'
    [ "$CLIPPY_ERR" -ne 0 ] && echo_err 'Please fix the issues from `cargo +nightly clippy -- -D warnings` first.'
    return $(( FMT_ERR || CLIPPY_ERR ))
}
export -f run_linters

# usage: cargo_asan [SUBCOMMAND] [OPTIONS] [-- <args>...]
# example: cargo_asan test --release TEST_NAME -- --skip SKIPPED
# NOTE: sanitizer documentation at https://doc.rust-lang.org/beta/unstable-book/compiler-flags/sanitizer.html
cargo_asan() {
    local SUBCOMMAND=$1; shift
    local TARGET_TRIPLE=$(rustc -vV | sed -n 's|host: ||p')
    RUSTFLAGS="-Z sanitizer=address" \
        # ASAN_OPTIONS="detect_leaks=1" \
        RUSTDOCFLAGS="-Z sanitizer=address" \
        cargo +$RUST_NIGHTLY $SUBCOMMAND -Z build-std --target $TARGET_TRIPLE "$@"
}
export -f cargo_asan

cargo_tsan() {
    local SUBCOMMAND=$1; shift
    local TARGET_TRIPLE=$(rustc -vV | sed -n 's|host: ||p')
    RUSTFLAGS="-Z sanitizer=thread" \
        TSAN_OPTIONS="suppressions=suppress_tsan.txt" \
        RUSTDOCFLAGS="-Z sanitizer=thread" \
        RUST_TEST_THREADS=1 \
        cargo +$RUST_NIGHTLY $SUBCOMMAND -Z build-std --target $TARGET_TRIPLE "$@"
}
export -f cargo_tsan

# usage: _run_tests_with CARGO [OPTIONS]
# example: _run_tests_with cargo_tsan --release
# Echos number of failed tests to stdout.
# Echos error message to stderr.
# Uses global variables TESTS, TIMEOUT.
# [OPTIONS] must not contain " -- " (cargo options only).
_run_tests_with() {
    local CARGO=$1; shift
    local MSGS # https://mywiki.wooledge.org/BashPitfalls#local_var.3D.24.28cmd.29
    MSGS=$($CARGO test --no-run "$@" 2>&1)
    if [ $? -ne 0 ]; then
        echo_err "Build failed! Error message:"
        echo_err "${MSGS}"
        echo_err "--------------------------------------------------------------------------------"
        echo ${#TESTS[@]} # failed all tests
        exit 1
    fi

    local FAILED=0
    for TEST in "${TESTS[@]}"; do
        local TEST_CMD="$CARGO test $* $TEST"
        timeout ${TIMEOUT:-20s} bash -c "$TEST_CMD" 1>&2
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
    # "cargo --release" should be split into "cargo" and "--release"
    local IFS=' '
    _run_tests_with $RUNNER
}
export -f run_tests
