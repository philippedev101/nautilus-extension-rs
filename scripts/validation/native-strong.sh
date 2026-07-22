#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/native-validation}"
export NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4=1
export G_DEBUG="${G_DEBUG:-fatal-warnings,fatal-criticals}"
unset NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG

perl scripts/validation/check-api-surface.pl --require-gir
bash scripts/validation/sys-abi-probes.sh
bash scripts/validation/check-example-symbols.sh
bash scripts/validation/gmodule-load-examples.sh
bash scripts/prek/cargo-quality.sh test
