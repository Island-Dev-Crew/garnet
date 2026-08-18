#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
    echo "usage: replay_memcheck.sh PHASE HEAD BINARY OUTPUT_DIRECTORY" >&2
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
overall_status=0

for teardown_case in working-clear episodic-drop semantic-drop; do
    output_path="$output_directory/$teardown_case-1024.memcheck.txt"
    replacement_path="$output_path.new"
    if [[ -e "$replacement_path" ]]; then
        echo "refusing stale replacement path: $replacement_path" >&2
        exit 73
    fi

    set +e
    GARNET_HEAD="$head" \
        valgrind \
        --tool=memcheck \
        --leak-check=full \
        --show-leak-kinds=all \
        --error-exitcode=99 \
        "$binary" "$teardown_case" 1024 \
        >"$replacement_path" 2>&1
    capture_status=$?
    set -e

    mv -f -- "$replacement_path" "$output_path"
    echo "RESULT phase=$phase case=$teardown_case size=1024 exit=$capture_status path=$output_path"
    if [[ "$capture_status" -ne 0 ]]; then
        overall_status="$capture_status"
    fi
done

exit "$overall_status"
