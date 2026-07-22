#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

if ! cargo semver-checks --version >/dev/null 2>&1; then
    echo "semver: cargo-semver-checks is not installed" >&2
    echo "semver: enter the Nix shell or install cargo-semver-checks" >&2
    exit 1
fi

baseline_args=()
baseline_count=0

if [ -n "${NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_VERSION:-}" ]; then
    baseline_args+=(--baseline-version "$NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_VERSION")
    baseline_count=$((baseline_count + 1))
fi

if [ -n "${NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_REV:-}" ]; then
    baseline_args+=(--baseline-rev "$NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_REV")
    baseline_count=$((baseline_count + 1))
fi

if [ -n "${NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_ROOT:-}" ]; then
    baseline_args+=(--baseline-root "$NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_ROOT")
    baseline_count=$((baseline_count + 1))
fi

if [ "$baseline_count" -gt 1 ]; then
    echo "semver: choose only one baseline source" >&2
    exit 2
fi

if [ "$baseline_count" -eq 0 ]; then
    cat <<'MESSAGE'
semver: skipped because no baseline was configured.
semver: set one of:
  NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_VERSION
  NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_REV
  NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_ROOT

This is expected before the first Nautilus API 4 release is published.
MESSAGE
    exit 0
fi

if [ "${NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4:-}" != "1" ]; then
    if ! pkg-config --atleast-version=43 libnautilus-extension-4 >/dev/null 2>&1 \
        && ! pkg-config --atleast-version=43 libnautilus-extension >/dev/null 2>&1; then
        export NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG=1
    fi
fi

release_args=()
if [ -n "${NAUTILUS_EXTENSION_RS_SEMVER_RELEASE_TYPE:-}" ]; then
    release_args+=(--release-type "$NAUTILUS_EXTENSION_RS_SEMVER_RELEASE_TYPE")
fi

for package in nautilus-extension-sys nautilus-extension; do
    cargo semver-checks -p "$package" "${baseline_args[@]}" "${release_args[@]}"
done
