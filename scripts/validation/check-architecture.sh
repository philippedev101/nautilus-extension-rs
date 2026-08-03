#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

max_lines=1000
failed=0

while IFS= read -r -d '' path; do
    lines="$(wc -l < "$path")"
    if [ "$lines" -gt "$max_lines" ]; then
        echo "architecture: $path has $lines lines; limit is $max_lines" >&2
        failed=1
    fi
done < <(
    find . \
        -path './.git' -prune -o \
        -path './target' -prune -o \
        -path './fuzz/target' -prune -o \
        -type f -name '*.rs' -print0
)

while IFS= read -r dir; do
    count="$(
        find "$dir" -maxdepth 1 -type f -name '*.rs' ! -name 'build.rs' | wc -l
    )"
    if [ "$count" -eq 1 ]; then
        echo "architecture: $dir contains exactly one Rust source file" >&2
        failed=1
    fi
done < <(
    find . \
        -path './.git' -prune -o \
        -path './target' -prune -o \
        -path './fuzz/target' -prune -o \
        -type d -print
)

if [ "$failed" -ne 0 ]; then
    exit 1
fi

echo "architecture: Rust source files are under ${max_lines} lines and source directories are not singleton"
