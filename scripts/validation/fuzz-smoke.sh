#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

if ! command -v cargo-fuzz >/dev/null 2>&1; then
    echo "cargo-fuzz is required; install it or run through a Nix shell that provides it" >&2
    exit 1
fi

toolchain="${RUSTUP_TOOLCHAIN:-nightly}"

has_asan_runtime() {
    local rustc="$1"
    local sysroot
    local target="${CARGO_BUILD_TARGET:-x86_64-unknown-linux-gnu}"

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

fuzz_cargo="${NAUTILUS_EXTENSION_RS_FUZZ_CARGO:-}"
fuzz_rustc="${NAUTILUS_EXTENSION_RS_FUZZ_RUSTC:-}"
needs_bootstrap="${NAUTILUS_EXTENSION_RS_FUZZ_RUSTC_BOOTSTRAP:-}"

if [ -z "$fuzz_cargo" ] || [ -z "$fuzz_rustc" ]; then
    fuzz_cargo="$(rustup which --toolchain "$toolchain" cargo 2>/dev/null || true)"
    fuzz_rustc="$(rustup which --toolchain "$toolchain" rustc 2>/dev/null || true)"
    if [ -n "$fuzz_cargo" ] && [ -n "$fuzz_rustc" ] && ! "$fuzz_cargo" --version >/dev/null 2>&1; then
        fuzz_cargo=""
        fuzz_rustc=""
    fi
fi

if [ -z "$fuzz_cargo" ] || [ -z "$fuzz_rustc" ]; then
    if bootstrap_toolchain="$(find_nix_bootstrap_toolchain)"; then
        fuzz_cargo="$(printf '%s\n' "$bootstrap_toolchain" | sed -n '1p')"
        fuzz_rustc="$(printf '%s\n' "$bootstrap_toolchain" | sed -n '2p')"
        needs_bootstrap=1
    fi
fi

if [ -z "$fuzz_cargo" ] || [ -z "$fuzz_rustc" ]; then
    echo "an executable Rust toolchain with ASan runtime is required for cargo-fuzz" >&2
    echo "on NixOS, run this inside the repository nix-shell or install a rust-overlay nightly" >&2
    exit 1
fi
if ! "$fuzz_cargo" --version >/dev/null 2>&1 || ! "$fuzz_rustc" --version >/dev/null 2>&1; then
    echo "selected fuzzing Rust toolchain cannot execute" >&2
    echo "cargo: $fuzz_cargo" >&2
    echo "rustc: $fuzz_rustc" >&2
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

export RUSTC="$fuzz_rustc"
export PATH="$(dirname "$fuzz_cargo"):$PATH"
if [ "$needs_bootstrap" = "1" ]; then
    export RUSTC_BOOTSTRAP=1
fi
default_lsan_options="suppressions=$repo_root/scripts/validation/lsan.supp:print_suppressions=0"
if [ -n "${LSAN_OPTIONS:-}" ]; then
    export LSAN_OPTIONS="$default_lsan_options:$LSAN_OPTIONS"
else
    export LSAN_OPTIONS="$default_lsan_options"
fi

if [ "${NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4:-}" != "1" ]; then
    if ! pkg-config --atleast-version=43 libnautilus-extension-4 >/dev/null 2>&1 \
        && ! pkg-config --atleast-version=43 libnautilus-extension >/dev/null 2>&1; then
        export NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG=1
    fi
fi

targets=(
    wrapper_inputs
    provider_outputs
)

for target in "${targets[@]}"; do
    "$fuzz_cargo" fuzz run "$target" -- -runs="${FUZZ_RUNS:-256}"
done
