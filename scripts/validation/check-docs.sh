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
# The MSRV is declared in two manifests and pinned in the CI job that proves it.
# Derive it from one place and fail if the other two disagree.
msrv="$(sed -n 's/^rust-version = "\(.*\)"$/\1/p' nautilus-extension/Cargo.toml)"

if [ -z "$msrv" ]; then
    echo "docs: nautilus-extension/Cargo.toml does not declare rust-version" >&2
    exit 1
fi

if ! grep -Fq "rust-version = \"$msrv\"" nautilus-extension-sys/Cargo.toml; then
    echo "docs: nautilus-extension-sys declares a different rust-version than $msrv" >&2
    exit 1
fi

if ! grep -Fq "dtolnay/rust-toolchain@$msrv" .github/workflows/validation.yml; then
    echo "docs: no CI job pins the declared rust-version $msrv" >&2
    exit 1
fi
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
