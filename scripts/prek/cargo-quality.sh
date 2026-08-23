#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

CARGO_CMD=(rustup run stable cargo)
export RUSTC="$(rustup which --toolchain stable rustc)"
export RUSTDOC="$(rustup which --toolchain stable rustdoc)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/prek}"

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

has_nautilus4_pkg_config() {
    pkg-config --atleast-version=43 libnautilus-extension-4 >/dev/null 2>&1 \
        || pkg-config --atleast-version=43 libnautilus-extension >/dev/null 2>&1
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
elif ! has_nautilus4_pkg_config; then
    export NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG=1
fi

run_with_rust_warnings_denied() {
    export RUSTFLAGS="${RUSTFLAGS:-} -Dwarnings"
    "${CARGO_CMD[@]}" "$@"
}

run_with_rustdoc_warnings_denied() {
    export RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -Dwarnings"
    "${CARGO_CMD[@]}" "$@"
}

# A GLib critical means a wrapper handed GLib something it should have rejected
# first, such as a stub GType in a build without the native Nautilus library.
# The safe API often still answers correctly, so the run has to abort instead.
run_tests_with_glib_criticals_fatal() {
    export G_DEBUG="${G_DEBUG:-fatal-warnings,fatal-criticals}"
    run_with_rust_warnings_denied "$@"
}

case "${1:-}" in
    fmt)
        "${CARGO_CMD[@]}" fmt -- --check
        ;;
    clippy)
        run_with_rust_warnings_denied clippy --all-targets -- -D warnings
        ;;
    test)
        run_tests_with_glib_criticals_fatal test --all-targets
        ;;
    doctest)
        run_tests_with_glib_criticals_fatal test --doc -p nautilus-extension
        ;;
    doc)
        run_with_rustdoc_warnings_denied doc --no-deps
        ;;
    *)
        echo "usage: $0 {fmt|clippy|test|doctest|doc}" >&2
        exit 2
        ;;
esac
