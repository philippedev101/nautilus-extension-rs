#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

if [ "${NAUTILUS_EXTENSION_RS_REAL_SMOKE:-}" != "1" ]; then
    cat >&2 <<'EOF'
Set NAUTILUS_EXTENSION_RS_REAL_SMOKE=1 to run this GUI smoke test.
It will build the example extensions, copy them to NAUTILUS_EXTENSION_RS_SMOKE_INSTALL_DIR
or to pkg-config's libnautilus-extension-4 extensiondir when that directory is writable,
then launch Nautilus. Run this only in a disposable desktop session.
EOF
    exit 2
fi

if ! command -v nautilus >/dev/null 2>&1; then
    echo "nautilus executable not found" >&2
    exit 1
fi

pkg=libnautilus-extension-4
if ! pkg-config --exists "$pkg" >/dev/null 2>&1; then
    pkg=libnautilus-extension
fi

extension_dir="${NAUTILUS_EXTENSION_RS_SMOKE_INSTALL_DIR:-$(pkg-config --variable=extensiondir "$pkg")}"
if [ -z "$extension_dir" ]; then
    echo "could not determine Nautilus extension directory" >&2
    exit 1
fi
if [ ! -d "$extension_dir" ] || [ ! -w "$extension_dir" ]; then
    echo "extension directory is not writable: $extension_dir" >&2
    echo "set NAUTILUS_EXTENSION_RS_SMOKE_INSTALL_DIR to a writable disposable directory" >&2
    exit 1
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/real-nautilus-smoke}"
export NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4=1
unset NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG

bash scripts/validation/check-example-symbols.sh

for so in "$CARGO_TARGET_DIR"/debug/examples/lib*_provider.so; do
    cp "$so" "$extension_dir/"
done

G_DEBUG="${G_DEBUG:-fatal-warnings,fatal-criticals}" nautilus --quit >/dev/null 2>&1 || true
G_DEBUG="${G_DEBUG:-fatal-warnings,fatal-criticals}" nautilus "${NAUTILUS_EXTENSION_RS_SMOKE_URI:-/tmp}"
