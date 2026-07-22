#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "usage: $0 [--allow-dirty] [--skip-publish-dry-run] [--strict]" >&2
}

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

allow_dirty=0
skip_publish_dry_run=0
strict=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --allow-dirty)
            allow_dirty=1
            ;;
        --skip-publish-dry-run)
            skip_publish_dry_run=1
            ;;
        --strict)
            strict=1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            usage
            exit 2
            ;;
    esac
    shift
done

package_args=()
publish_args=(--dry-run)
if [ "$allow_dirty" -eq 1 ]; then
    package_args+=(--allow-dirty)
    publish_args+=(--allow-dirty)
fi

mkdir -p target/package-validation
sys_version="$(
    sed -n 's/^version = "\(.*\)"/\1/p' nautilus-extension-sys/Cargo.toml | head -n 1
)"

for package in nautilus-extension-sys nautilus-extension; do
    rustup run stable cargo package --list -p "$package" "${package_args[@]}" \
        > "target/package-validation/${package}.files"
done

if [ "$skip_publish_dry_run" -eq 1 ]; then
    echo "package: skipped cargo publish --dry-run"
    exit 0
fi

rustup run stable cargo publish -p nautilus-extension-sys "${publish_args[@]}"

wrapper_log="target/package-validation/nautilus-extension.publish-dry-run.log"
if ! rustup run stable cargo publish -p nautilus-extension "${publish_args[@]}" \
    >"$wrapper_log" 2>&1; then
    cat "$wrapper_log" >&2
    if [ "$strict" -eq 0 ] \
        && grep -F "failed to select a version for the requirement \`nautilus-extension-sys = \"^$sys_version\"\`" "$wrapper_log" >/dev/null; then
        cat >&2 <<'MESSAGE'
package: deferred nautilus-extension publish dry-run.

The matching nautilus-extension-sys version is not available in the registry
yet. This is expected before publishing the sys crate. After publishing
nautilus-extension-sys and waiting for the registry index, rerun with --strict
before publishing nautilus-extension.
MESSAGE
        exit 0
    fi

    cat >&2 <<'MESSAGE'
package: nautilus-extension publish dry-run failed.

If the failure says the matching nautilus-extension-sys version is not
available in the registry, publish nautilus-extension-sys first, wait for the
registry index to update, and rerun this script before publishing the safe
wrapper crate.
MESSAGE
    exit 1
fi
