#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
    echo "usage: replay_callgrind.sh PHASE HEAD BINARY OUTPUT_DIRECTORY" >&2
    exit 64
fi

phase="$1"
head="$2"
binary="$3"
output_directory="$4"

case "$phase" in
    before | after) ;;
    *)
        echo "PHASE must be before or after" >&2
        exit 64
        ;;
esac

if [[ ! -x "$binary" ]]; then
    echo "BINARY is not executable: $binary" >&2
    exit 66
fi

mkdir -p "$output_directory"

for teardown_case in working-clear episodic-drop semantic-drop; do
    for input_size in 256 512 1024; do
        output_path="$output_directory/$teardown_case-$input_size.callgrind"
        replacement_path="$output_path.new"
        if [[ -e "$replacement_path" ]]; then
            echo "refusing stale replacement path: $replacement_path" >&2
            exit 73
        fi

        GARNET_HEAD="$head" GARNET_LANE2C_CALLGRIND=1 \
            valgrind \
            --tool=callgrind \
            --instr-atstart=no \
            --error-exitcode=97 \
            --callgrind-out-file="$replacement_path" \
            "$binary" "$teardown_case" "$input_size"

        mv -f -- "$replacement_path" "$output_path"
        instructions="$(
            awk '$1 == "summary:" { print $2 }' "$output_path"
        )"
        echo "RESULT phase=$phase case=$teardown_case size=$input_size instructions=$instructions"
    done
done
