# Versioning Policy

This workspace contains two crates:

1. `nautilus-extension-sys` for raw FFI bindings.
2. `nautilus-extension` for safe-ish Rust wrappers and provider helpers.

Both crates use matching versions. The first Nautilus API 4 / GTK 4-era line is
`0.9.x`. It is intentionally source-incompatible with the older `0.8.x`
Nautilus 3 / GTK 3-era public API.

Because the crates are still pre-1.0, semver-breaking Rust API changes use a
minor version bump. Patch releases are reserved for compatible fixes,
documentation improvements, validation improvements, and additions that do not
break existing `0.9.x` extension crates.

## Compatibility Rules

1. Nautilus API 4 is the primary supported ABI.
2. Nautilus 3 provider APIs are not preserved behind compatibility shims.
3. `nautilus-extension` may add Rust-style aliases for APIs that still keep
   Nautilus-style names for familiarity with the C ABI.
4. `nautilus-extension-sys` tracks the C ABI names exactly.
5. When Nautilus changes API 4.x, update `docs/API_TRACKING.md` and the static
   API surface checker in the same change.
6. The minimum supported Rust version is declared as `rust-version` in both
   manifests and pinned by the CI job that compiles against it. Raising it is a
   breaking change for downstream crates, so it goes with a minor bump and a
   `CHANGELOG.md` entry. `check-docs.sh` fails if the two manifests and the CI
   pin disagree, so the bump is a single edit followed by the checks.

## Semver Checks

Run `scripts/validation/semver.sh` before publishing compatible releases.
After `0.9.0` is published, use the latest published compatible version as the
baseline:

```sh
NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_VERSION=0.9.0 \
  bash scripts/validation/semver.sh
```

For local comparisons against a branch or tag:

```sh
NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_REV=origin/gtk4 \
  bash scripts/validation/semver.sh
```

If a breaking change is intentional, bump the minor version and document the
change in `CHANGELOG.md`.
