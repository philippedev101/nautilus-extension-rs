#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

required_files=(
    docs/API_TRACKING.md
    docs/API_NAMING.md
    docs/ARCHITECTURE.md
    docs/FFI_AUDIT.md
    docs/NAUTILUS_3_MIGRATION.md
    docs/RELEASE.md
    docs/VERSIONING.md
    nautilus-extension/README.md
    nautilus-extension/examples/README.md
    nautilus-extension/examples/column_provider/README.md
    nautilus-extension/examples/menu_provider/README.md
    nautilus-extension/examples/properties_model_provider/README.md
)

for path in "${required_files[@]}"; do
    if [ ! -s "$path" ]; then
        echo "docs: missing required documentation file: $path" >&2
        exit 1
    fi
done

grep -F '[package.metadata.docs.rs]' nautilus-extension/Cargo.toml >/dev/null
grep -F '[package.metadata.docs.rs]' nautilus-extension-sys/Cargo.toml >/dev/null
grep -F 'readme = "README.md"' nautilus-extension/Cargo.toml >/dev/null
grep -F 'readme = "README.md"' nautilus-extension-sys/Cargo.toml >/dev/null
grep -F 'links = "nautilus-extension"' nautilus-extension-sys/Cargo.toml >/dev/null
grep -F 'rust-version = "1.92"' nautilus-extension/Cargo.toml >/dev/null
grep -F 'rust-version = "1.92"' nautilus-extension-sys/Cargo.toml >/dev/null
grep -F 'DOCS_RS' nautilus-extension-sys/build.rs >/dev/null

for example in column_provider menu_provider properties_model_provider; do
    grep -F "cargo build --example $example" "nautilus-extension/examples/$example/README.md" >/dev/null
    grep -F 'extensions-4' "nautilus-extension/examples/$example/README.md" >/dev/null
done

for doc in \
    docs/API_TRACKING.md \
    docs/API_NAMING.md \
    docs/FFI_AUDIT.md \
    docs/NAUTILUS_3_MIGRATION.md \
    docs/RELEASE.md \
    docs/VERSIONING.md; do
    grep -F 'Nautilus API 4' "$doc" >/dev/null
done

echo "docs: required documentation files and metadata are present"
