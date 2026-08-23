# nautilus-extension-sys

FFI bindings to the Nautilus API 4 extension library.

The build script probes `libnautilus-extension-4` first and then
`libnautilus-extension` with pkg-config, requiring a Nautilus 43+ package for
either name. It links against the Nautilus extension library through the
unversioned linker name `nautilus-extension`.

Most users should depend on the `nautilus-extension` crate instead of this raw
sys crate.

`NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG=1` exists only for non-linking
syntax checks and pure Rust unit tests in minimal environments. It cannot
produce a loadable extension unless the Nautilus API 4 library is actually
available to the linker.

In that configuration the C symbols cannot be referenced at all, so `src/unlinked.rs`
expands the same Nautilus API declarations into stubs with identical signatures
that return a neutral value. `NATIVE_API_AVAILABLE` reports which of the two
builds this is, so callers can branch at run time instead of duplicating
themselves behind a cfg.
