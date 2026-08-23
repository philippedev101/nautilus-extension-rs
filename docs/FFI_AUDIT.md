# FFI Panic and Unsafe Audit

This audit applies to the Nautilus API 4 Rust wrapper and sys crates.

Nautilus loads extension shared objects in-process. Rust panics must not unwind
through Nautilus or GLib C frames. Raw pointer use must stay local to wrapper
constructors, the `GObjectProperties` and `NautilusObject` accessor traits, the
`from_raw_*` and `into_raw` boundary functions, and generated trampolines.

## Builds Without the Native Library

On docs.rs, and on hosts where `NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG`
allows a build without the Nautilus development package, the C symbols cannot be
referenced. `nautilus-extension-sys/src/unlinked.rs` then substitutes stubs with
the same signatures that ignore their arguments and return a neutral value: a
null pointer, `G_TYPE_INVALID`, `FALSE`, or `NautilusOperationFailed`. Those
values flow through the ordinary null checks in the wrappers, so the safe API
reports `None`, `false`, and `OperationResult::Failed` without a second code
path. Nothing in the wrapper crates is compiled conditionally on that build
mode; the public `NATIVE_API_AVAILABLE` constant is the run-time signal.

## C ABI Entry Points

The `nautilus_module!` macro exports the required Nautilus symbols:

- `nautilus_module_initialize`
- `nautilus_module_list_types`
- `nautilus_module_shutdown`

`nautilus_module_initialize` and `nautilus_module_shutdown` catch panics before
returning to C. If registration panics, the exported type list is cleared and
provider state is reset.

`nautilus_module_list_types` does not call extension-provided code. It only
reads the stored type list and handles poisoned locks by returning an empty
list.

## Provider Callback Boundaries

Generated provider trampolines catch panics around extension trait methods:

- `ColumnProvider::get_columns`
- `InfoProvider::update_file_info_full`
- `InfoProvider::cancel_update`
- `MenuProvider::get_file_items`
- `MenuProvider::get_file_items_full`
- `MenuProvider::get_background_items`
- `MenuProvider::get_background_items_full`
- `PropertiesModelProvider::get_models`
- `FileInfoImpl` interface methods
- `MenuItemActivate::activate`
- `MenuItem::on_activate` callbacks
- signal callback destroy functions

Panic tests live in the provider modules and in `nautilus_module` test support.
They exercise the generated trampolines directly without loading Nautilus.

## Asynchronous InfoProvider Boundary

`UpdateCompletion::complete_with` schedules completion work back onto Nautilus'
main context. The scheduled closure catches panics before invoking Nautilus'
completion callback. If scheduling fails, completion state is rolled back so a
caller can retry or fail explicitly.

Extension authors should move plain Rust data to worker threads, not `FileInfo`
objects. The `complete_with` closure is the supported place to mutate
`FileInfo`.

## Ownership Rules

The safe wrapper layer follows these transfer conventions:

- `from_raw_borrowed` adds a GObject reference and returns an owned Rust
  wrapper.
- `from_raw_full` takes ownership of a full-transfer reference.
- `into_raw` transfers ownership back to the caller and prevents `Drop` from
  unreffing the pointer.
- `FileInfoList` and `MenuItemList` free Nautilus-owned `GList` values with the
  matching Nautilus free function.
- Generic helper lists created only for interface calls are freed with
  `g_list_free`.
- GLib strings returned by Nautilus are consumed with `g_free` through
  `take_glib_string`.
- The sys crate encodes string transfer in the return type: a getter declared
  `*mut c_char` is transfer-full and its result is freed, a getter declared
  `*const c_char` is transfer-none and its result is only copied. The
  `NautilusObject::owned_string` and `borrowed_string` accessors rely on that
  split, so a new binding must be declared with the pointer constness that
  matches the Nautilus annotation.

## Linting Policy

The public crates enable:

- `missing_docs`
- `rustdoc::broken_intra_doc_links`
- `unsafe_op_in_unsafe_fn`

`unsafe_op_in_unsafe_fn` keeps unsafe operations visible even inside unsafe
functions. Some callback and GObject integration code necessarily remains
unsafe, but new unsafe operations should be accompanied by a precise local
reason.

Accessors are not the place for that. A new object wrapper implements
`GObjectProperties` or `NautilusObject` and gets a safe accessor surface from
it, so the wrapper states its pointer invariant once rather than once per
method.

## Validation

Run these checks before release:

```sh
bash scripts/validation/all-fast.sh
nix-shell --run 'bash scripts/validation/native-strong.sh'
bash scripts/validation/asan-lsan.sh
bash scripts/validation/valgrind.sh
bash scripts/validation/fuzz-smoke.sh
```

The sanitizer suppression file is limited to known Nautilus allocation
behavior observed while loading example modules. New leaks in Rust-owned
objects should be fixed rather than suppressed.

The unit test run sets `G_DEBUG=fatal-warnings,fatal-criticals`. A wrapper that
hands GLib something it should have rejected first, such as a stub GType in an
unlinked build or a null object, often still returns the right answer, so the
GLib critical is the only signal. Making it fatal turns that into a test
failure, and the accessor null-guard tests depend on it.

The Valgrind smoke script fails on invalid reads, invalid writes, use of
uninitialized values, and other memory errors. It intentionally does not fail on
small process-exit leaks from GLib/GObject/Nautilus type registration and module
loading, because those are outside the Rust wrapper ownership model and are
reported by Valgrind even when the example module loads and shuts down cleanly.
