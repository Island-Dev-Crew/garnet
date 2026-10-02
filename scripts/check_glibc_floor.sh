#!/bin/sh
# Fail when a binary imports a glibc symbol version newer than the floor.
#
#   scripts/check_glibc_floor.sh <binary> <floor, e.g. 2.34>
#
# Debian 12, Ubuntu 22.04 and RHEL 9 ship glibc 2.36, 2.35 and 2.34, so the
# Linux release assets use a 2.34 floor. Finding no GLIBC_ versions at all is
# a failure too: it means objdump did not read the binary.
set -eu

binary="$1"
floor="$2"

by_version() {
    sort -t. -k1,1n -k2,2n -k3,3n
}

versions="$(objdump -T "$binary" | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/^GLIBC_//' | by_version | uniq)" || true
if [ -z "$versions" ]; then
    echo "error: no GLIBC_ symbol versions found in $binary" >&2
    exit 1
fi
newest="$(printf '%s\n' "$versions" | tail -n 1)"
echo "newest glibc symbol version in $binary: GLIBC_$newest (floor GLIBC_$floor)"
top="$(printf '%s\n%s\n' "$newest" "$floor" | by_version | tail -n 1)"
if [ "$top" != "$floor" ]; then
    echo "error: $binary imports GLIBC_$newest, newer than the GLIBC_$floor floor" >&2
    exit 1
fi
