# Release Checklist

Use this checklist for both crates. The sys crate must be published before the
safe wrapper crate because `nautilus-extension` depends on the matching
`nautilus-extension-sys` version.

## Before Release

1. Confirm both crate versions match in:
   - `nautilus-extension/Cargo.toml`
   - `nautilus-extension-sys/Cargo.toml`
   - `Cargo.lock`
2. Update `CHANGELOG.md`.
3. Confirm the versioning impact described in `docs/VERSIONING.md`.
4. Run fast validation:

```sh
bash scripts/validation/all-fast.sh
```

5. Run native validation on a machine or dev shell with Nautilus API 4 headers:

```sh
nix-shell --run 'bash scripts/validation/native-strong.sh'
```

6. Run package validation:

```sh
bash scripts/validation/package.sh
```

For a dirty local branch, use:

```sh
bash scripts/validation/package.sh --allow-dirty
```

Before `nautilus-extension-sys` is published, the wrapper crate dry-run is
reported as deferred because crates.io cannot resolve the matching sys crate
version yet. After publishing `nautilus-extension-sys` and waiting for the
registry index, rerun strict validation before publishing the safe wrapper:

```sh
bash scripts/validation/package.sh --strict
```

7. Run semver checks when a compatible baseline exists:

```sh
NAUTILUS_EXTENSION_RS_SEMVER_BASELINE_VERSION=0.9.0 \
  bash scripts/validation/semver.sh
```

## Manual Nautilus Smoke Test

1. Build the example shared objects:

```sh
cargo build --example column_provider
cargo build --example menu_provider
cargo build --example properties_model_provider
```

2. Find the distro extension directory:

```sh
pkg-config --variable=extensiondir libnautilus-extension-4
pkg-config --variable=extensiondir libnautilus-extension
```

3. Copy one example `.so` into the Nautilus API 4 extension directory.
4. Restart Nautilus:

```sh
nautilus -q
```

5. Verify:
   - the column example exposes the `Demo Status` list-view column;
   - the menu example adds neutral context-menu items;
   - the properties example adds a `Rust Demo` properties section.

## Publish Order

1. Publish `nautilus-extension-sys`.
2. Wait for crates.io to index the new sys crate version.
3. Run `bash scripts/validation/package.sh --strict`.
4. Publish `nautilus-extension`.
5. Tag the release.
6. Push the tag and release notes.
