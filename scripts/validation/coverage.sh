#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "cargo-llvm-cov is required; install it or run through a Nix shell that provides it" >&2
    exit 1
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/coverage}"

prepend_pkg_config_path() {
    local dir="$1"

    case ":${PKG_CONFIG_PATH:-}:" in
        *":$dir:"*) ;;
        *)
            if [ -n "${PKG_CONFIG_PATH:-}" ]; then
                export PKG_CONFIG_PATH="$dir:$PKG_CONFIG_PATH"
            else
                export PKG_CONFIG_PATH="$dir"
            fi
            ;;
    esac
}

discover_nix_pkg_config_dir() {
    local pattern="$1"
    local required="$2"
    local dir

    for dir in $pattern; do
        if [ -f "$dir/$required" ]; then
            printf '%s\n' "$dir"
            return 0
        fi
    done

    return 1
}

if ! pkg-config --exists glib-2.0 gobject-2.0 gio-2.0 >/dev/null 2>&1; then
    if glib_dir="$(discover_nix_pkg_config_dir "/nix/store/*-glib-*-dev/lib/pkgconfig" "gio-2.0.pc")"; then
        prepend_pkg_config_path "$glib_dir"
    fi
fi

if ! pkg-config --exists zlib >/dev/null 2>&1; then
    if zlib_dir="$(discover_nix_pkg_config_dir "/nix/store/*-zlib-*-dev/lib/pkgconfig" "zlib.pc")"; then
        prepend_pkg_config_path "$zlib_dir"
    elif zlib_dir="$(discover_nix_pkg_config_dir "/nix/store/*-zlib-*-dev/share/pkgconfig" "zlib.pc")"; then
        prepend_pkg_config_path "$zlib_dir"
    fi
fi

if [ "${NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4:-}" = "1" ]; then
    unset NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG
elif ! pkg-config --atleast-version=43 libnautilus-extension-4 >/dev/null 2>&1 \
    && ! pkg-config --atleast-version=43 libnautilus-extension >/dev/null 2>&1; then
    export NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG=1
fi

if [ -z "${LLVM_COV:-}" ] && command -v llvm-cov >/dev/null 2>&1; then
    export LLVM_COV="$(command -v llvm-cov)"
fi

if [ -z "${LLVM_PROFDATA:-}" ] && command -v llvm-profdata >/dev/null 2>&1; then
    export LLVM_PROFDATA="$(command -v llvm-profdata)"
fi

cargo llvm-cov --workspace --all-targets --html --output-dir target/coverage/html
echo "coverage report: target/coverage/html/html/index.html"
