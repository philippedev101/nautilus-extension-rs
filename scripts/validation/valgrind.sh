#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

if ! command -v valgrind >/dev/null 2>&1; then
    echo "valgrind is required for this validation" >&2
    exit 1
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/valgrind-validation}"
export NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4=1
unset NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG

bash scripts/validation/gmodule-load-examples.sh

examples=(
    column_provider
    menu_provider
    properties_model_provider
)

for example in "${examples[@]}"; do
    valgrind \
        --error-exitcode=125 \
        --leak-check=full \
        --errors-for-leak-kinds=none \
        --show-leak-kinds=definite,possible \
        "$CARGO_TARGET_DIR/validation/gmodule_harness" \
        "$CARGO_TARGET_DIR/debug/examples/lib${example}.so"
done
