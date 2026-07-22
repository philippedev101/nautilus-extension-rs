#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/docs-rs-validation}"
export DOCS_RS=1
export RUSTC="$(rustup which --toolchain stable rustc)"
export RUSTDOC="$(rustup which --toolchain stable rustdoc)"
export RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -Dwarnings --cfg docsrs"

rustup run stable cargo doc --no-deps -p nautilus-extension-sys
rustup run stable cargo doc --no-deps -p nautilus-extension
