#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

toolchain="${RUSTUP_TOOLCHAIN:-nightly}"
target="${CARGO_BUILD_TARGET:-x86_64-unknown-linux-gnu}"

has_asan_runtime() {
    local rustc="$1"
    local sysroot

    sysroot="$("$rustc" --print sysroot 2>/dev/null)" || return 1
    compgen -G "$sysroot/lib/rustlib/$target/lib/librustc*asan*.a" >/dev/null
}

find_nix_bootstrap_toolchain() {
    local rustc
    local cargo

    for rustc in /nix/store/*-rustc-bootstrap-*/bin/rustc; do
        [ -x "$rustc" ] || continue
        has_asan_runtime "$rustc" || continue

        for cargo in /nix/store/*-cargo-bootstrap-*/bin/cargo "$(dirname "$rustc")/cargo"; do
            [ -x "$cargo" ] || continue
            printf '%s\n%s\n' "$cargo" "$rustc"
            return 0
        done
    done

    return 1
}

asan_cargo="${NAUTILUS_EXTENSION_RS_SANITIZER_CARGO:-}"
asan_rustc="${NAUTILUS_EXTENSION_RS_SANITIZER_RUSTC:-}"
needs_bootstrap="${NAUTILUS_EXTENSION_RS_SANITIZER_RUSTC_BOOTSTRAP:-}"

if [ -z "$asan_cargo" ] || [ -z "$asan_rustc" ]; then
    asan_cargo="$(rustup which --toolchain "$toolchain" cargo 2>/dev/null || true)"
    asan_rustc="$(rustup which --toolchain "$toolchain" rustc 2>/dev/null || true)"
    if [ -n "$asan_cargo" ] && [ -n "$asan_rustc" ] && ! "$asan_cargo" --version >/dev/null 2>&1; then
        asan_cargo=""
        asan_rustc=""
    fi
fi

if [ -z "$asan_cargo" ] || [ -z "$asan_rustc" ]; then
    if bootstrap_toolchain="$(find_nix_bootstrap_toolchain)"; then
        asan_cargo="$(printf '%s\n' "$bootstrap_toolchain" | sed -n '1p')"
        asan_rustc="$(printf '%s\n' "$bootstrap_toolchain" | sed -n '2p')"
        needs_bootstrap=1
    fi
fi

if [ -z "$asan_cargo" ] || [ -z "$asan_rustc" ]; then
    echo "an executable Rust toolchain with ASan runtime is required for sanitizer validation" >&2
    echo "on NixOS, run this inside the repository nix-shell or install a rust-overlay nightly" >&2
    exit 1
fi
if ! "$asan_cargo" --version >/dev/null 2>&1 || ! "$asan_rustc" --version >/dev/null 2>&1; then
    echo "selected sanitizer Rust toolchain cannot execute" >&2
    echo "cargo: $asan_cargo" >&2
    echo "rustc: $asan_rustc" >&2
    exit 1
fi

prepend_library_path() {
    local dir="$1"

    [ -n "$dir" ] || return 0
    case ":${LD_LIBRARY_PATH:-}:" in
        *":$dir:"*) ;;
        *)
            if [ -n "${LD_LIBRARY_PATH:-}" ]; then
                export LD_LIBRARY_PATH="$dir:$LD_LIBRARY_PATH"
            else
                export LD_LIBRARY_PATH="$dir"
            fi
            ;;
    esac
}

for pkg in libnautilus-extension-4 libnautilus-extension gio-2.0 gobject-2.0 glib-2.0; do
    if pkg-config --exists "$pkg" >/dev/null 2>&1; then
        prepend_library_path "$(pkg-config --variable=libdir "$pkg")"
    fi
done
if command -v cc >/dev/null 2>&1; then
    libstdcxx="$(cc -print-file-name=libstdc++.so.6 2>/dev/null || true)"
    if [ -f "$libstdcxx" ]; then
        prepend_library_path "$(dirname "$libstdcxx")"
    fi
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/asan-lsan}"
export NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4="${NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4:-1}"
export G_DEBUG="${G_DEBUG:-fatal-warnings,fatal-criticals}"
export ASAN_OPTIONS="${ASAN_OPTIONS:-detect_leaks=1:halt_on_error=1}"
default_lsan_options="suppressions=$repo_root/scripts/validation/lsan.supp:print_suppressions=0"
if [ -n "${LSAN_OPTIONS:-}" ]; then
    export LSAN_OPTIONS="$default_lsan_options:$LSAN_OPTIONS"
else
    export LSAN_OPTIONS="$default_lsan_options"
fi
export RUSTFLAGS="${RUSTFLAGS:-} -Z sanitizer=address"
export RUSTC="$asan_rustc"
export PATH="$(dirname "$asan_cargo"):$PATH"
if [ "$needs_bootstrap" = "1" ]; then
    export RUSTC_BOOTSTRAP=1
fi

"$asan_cargo" test --target "$target" --all-targets -- --test-threads=1
