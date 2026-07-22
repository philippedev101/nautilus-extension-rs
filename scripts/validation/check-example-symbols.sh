#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/validation}"

bash scripts/validation/build-examples.sh

examples=(
    column_provider
    menu_provider
    properties_model_provider
)

symbols=(
    nautilus_module_initialize
    nautilus_module_list_types
    nautilus_module_shutdown
)

for example in "${examples[@]}"; do
    so="$CARGO_TARGET_DIR/debug/examples/lib${example}.so"
    if [ ! -f "$so" ]; then
        echo "missing built example shared object: $so" >&2
        exit 1
    fi

    for symbol in "${symbols[@]}"; do
        if ! nm -D --defined-only "$so" | awk '{print $NF}' | grep -Fx "$symbol" >/dev/null; then
            echo "$so does not export $symbol" >&2
            exit 1
        fi
    done
done

echo "example-symbols: all example modules export Nautilus entry points"
