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

# The build-mode cfg belongs to the sys crate alone. Everywhere else the two
# builds share one code path and branch on NATIVE_API_AVAILABLE at run time.
while IFS= read -r path; do
    case "$path" in
        ./nautilus-extension-sys/*) continue ;;
    esac
    echo "architecture: $path branches on nautilus_extension_rs_skip_link" >&2
    echo "architecture: use the NATIVE_API_AVAILABLE constant instead" >&2
    failed=1
done < <(
    grep -rl 'cfg(\(.*\)\?nautilus_extension_rs_skip_link' \
        --include='*.rs' \
        --exclude-dir=target \
        . 2>/dev/null
)

if [ "$failed" -ne 0 ]; then
    exit 1
fi

echo "architecture: Rust source files are under ${max_lines} lines, source directories are not singleton, and the build-mode cfg is confined to nautilus-extension-sys"
