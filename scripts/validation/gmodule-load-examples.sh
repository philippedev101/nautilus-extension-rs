#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

pkg=libnautilus-extension-4
if ! pkg-config --exists "$pkg" >/dev/null 2>&1; then
    pkg=libnautilus-extension
fi

if ! pkg-config --atleast-version=43 "$pkg" >/dev/null 2>&1; then
    echo "Nautilus API 4 development files were not found by pkg-config" >&2
    exit 1
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/native-validation}"
export NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4=1
unset NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG

bash scripts/validation/build-examples.sh

mkdir -p "$CARGO_TARGET_DIR/validation"
cc scripts/validation/gmodule_harness.c \
    -o "$CARGO_TARGET_DIR/validation/gmodule_harness" \
    $(pkg-config --cflags --libs "$pkg" gmodule-2.0 gio-2.0 gobject-2.0 glib-2.0)

examples=(
    column_provider
    menu_provider
    properties_model_provider
)

for example in "${examples[@]}"; do
    "$CARGO_TARGET_DIR/validation/gmodule_harness" \
        "$CARGO_TARGET_DIR/debug/examples/lib${example}.so"
done
