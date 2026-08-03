#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

bash scripts/prek/cargo-quality.sh fmt
bash scripts/validation/check-architecture.sh
bash scripts/validation/check-docs.sh
perl scripts/validation/check-api-surface.pl
bash scripts/prek/cargo-quality.sh clippy
bash scripts/prek/cargo-quality.sh test
bash scripts/prek/cargo-quality.sh doctest
bash scripts/prek/cargo-quality.sh doc
bash scripts/validation/docs-rs.sh
bash scripts/validation/check-example-symbols.sh
git diff --check
