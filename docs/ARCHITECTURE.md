# Architecture

Nautilus extension bindings are organized as two workspace crates:

* `nautilus-extension-sys` exposes raw Nautilus API 4 FFI types and functions.
* `nautilus-extension` exposes safe-ish provider traits, object wrappers, and
  module registration helpers.

Provider modules use directory modules when they have multiple responsibilities.
The preferred shape is:

* `mod.rs` for imports, facade exports, and hidden compatibility paths.
* `api.rs`, `model.rs`, or similarly named files for public Rust-facing types.
* `*_iface.rs`, `*_class.rs`, or `activation.rs` for C ABI trampolines and
  generated slot state.
* `tests.rs` for unit tests that belong to that provider surface.

Builds without the native Nautilus library are handled in one place. The
`nautilus_api!` macro in `nautilus-extension-sys/src/unlinked.rs` declares every
Nautilus C function once and expands it either into the linked `extern "C"`
block or into a stub of the same signature returning a neutral value. Wrapper
source must not branch on the `nautilus_extension_rs_skip_link` cfg; where the
two builds genuinely differ, branch on the public `NATIVE_API_AVAILABLE`
constant at run time, and in tests use the `require_native_api!` and
`require_unlinked_build!` guards.
`check-architecture.sh` fails if that cfg appears outside `nautilus-extension-sys`.

Rust source files should stay at or below 1000 lines. A directory that contains
Rust source should contain more than one Rust source file; split tests or
implementation details into sibling files instead of leaving singleton source
directories. Run `bash scripts/validation/check-architecture.sh` before
structural changes.
