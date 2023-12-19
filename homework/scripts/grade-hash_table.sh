#!/usr/bin/env bash
# set -e
set -uo pipefail
IFS=$'\n\t'

time {

# Imports library.
BASEDIR=$(dirname "$0")
source $BASEDIR/grade-utils.sh

run_linters || exit 1

lines=$(grep_skip_comment transmute "$BASEDIR"/../src/hash_table/{growable_array,split_ordered_list}.rs )
if [ -n "$lines" ]; then
    echo "transmute() is not allowed."
    echo "$lines"
    exit 1
fi

export RUST_TEST_THREADS=1

REPS=5
TEST_NAMES=(
    "stress_sequential"
    "lookup_concurrent"
    "insert_concurrent"
    "stress_concurrent"
    "log_concurrent"
)
RUNNERS=(
    "cargo --release"
    "cargo_asan --release"
    "cargo_tsan --release"
)
# timeout for each (TEST_NAME, RUNNER).
TIMEOUTS=(
    10s 10s 10s
    10s 10s 10s
    10s 10s 10s
    10s 25s 15s
    10s 25s 15s
)
# the index of the last failed test
growable_array_fail=${#TEST_NAMES[@]}
split_ordered_list_fail=${#TEST_NAMES[@]}

echo "1. Running tests..."
for r in "${!RUNNERS[@]}"; do
    RUNNER=${RUNNERS[r]}
    for t in "${!TEST_NAMES[@]}"; do
        TEST_NAME=${TEST_NAMES[t]}
        TIMEOUT=${TIMEOUTS[ t * ${#RUNNERS[@]} + r ]}
        # run only if no test has failed yet
        if [ $t -lt $growable_array_fail ]; then
            echo "Testing growable_array $TEST_NAME with $RUNNER, timeout $TIMEOUT..."
            TESTS=("--test growable_array $TEST_NAME")
            for ((i = 0; i < REPS; i++)); do
                if [ $(run_tests) -ne 0 ]; then
                    growable_array_fail=$t
                    break
                fi
            done
        fi
        if [ $t -lt $split_ordered_list_fail ]; then
            echo "Testing split_ordered_list $TEST_NAME with $RUNNER, timeout $TIMEOUT..."
            TESTS=("--test split_ordered_list $TEST_NAME")
            for ((i = 0; i < REPS; i++)); do
                if [ $(run_tests) -ne 0 ]; then
                    split_ordered_list_fail=$t
                    break
                fi
            done
        fi
    done
done

# # 2. Check uses of SeqCst
# # Don't give performance_score score if tests failed.
# growable_array_performance_ok=false
# split_ordered_list_performance_ok=false
# if [ $growable_array_fail -eq ${#TEST_NAMES[@]} ]; then
#     echo "2. Checking uses of SeqCst..."
#     lines=$(grep_skip_comment SeqCst "$BASEDIR"/../src/hash_table/growable_array.rs )
#     if [ -n "$lines" ]; then
#         echo_err "You used SeqCst in growable_array (and transitively in split_ordered_list)!"
#         echo "$lines"
#         # Give zero in this case, because split_ordered_list uses growable_array.
#     else
#         growable_array_performance_ok=true
#         if [ $split_ordered_list_fail -eq ${#TEST_NAMES[@]} ]; then
#             lines=$(grep_skip_comment SeqCst "$BASEDIR"/../src/hash_table/split_ordered_list.rs )
#             if [ -n "$lines" ]; then
#                 echo_err "You used SeqCst in split_ordered_list!"
#                 echo "$lines"
#             else
#                 split_ordered_list_performance_ok=true
#             fi
#         fi
#     fi
# fi

}

SCORES=( 0 5 10 20 40 70 )
SCORE=$(( SCORES[growable_array_fail] + SCORES[split_ordered_list_fail] ))
# if [ "$growable_array_performance_ok" = true ]; then
#     SCORE=$((SCORE + 20))
# fi
# if [ "$split_ordered_list_performance_ok" = true ]; then
#     SCORE=$((SCORE + 20))
# fi

# echo "Score: $SCORE / 180"
echo "Score: $SCORE / 140"
