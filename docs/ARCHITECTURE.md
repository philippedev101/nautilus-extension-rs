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

Rust source files should stay at or below 1000 lines. A directory that contains
Rust source should contain more than one Rust source file; split tests or
implementation details into sibling files instead of leaving singleton source
directories. Run `bash scripts/validation/check-architecture.sh` before
structural changes.
