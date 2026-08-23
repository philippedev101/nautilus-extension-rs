# nautilus-extension

Safe-ish Rust wrappers for the Nautilus API 4 extension ABI used by Nautilus 43
and newer.

This crate is for reusable Nautilus extension modules. Build downstream
extensions as `cdylib` crates and export the C ABI entry points with
`nautilus_module!`.

## Supported Provider APIs

- `ColumnProvider`
- `InfoProvider`, including asynchronous `update_file_info_full`,
  `PendingUpdate`, `UpdateCompletion`, `OperationHandle`, and cancellation
- `MenuProvider`
- `PropertiesModelProvider`
- advanced `FileInfoImpl` registration for Rust-backed `FileInfo` objects

GTK 3-era Nautilus APIs such as `LocationWidgetProvider`, GTK-widget property
pages, and menu callbacks with GTK window arguments are not part of this
Nautilus API 4 wrapper.

## Async Model

`InfoProvider` is the async-capable provider surface because Nautilus exposes an
operation handle, completion callback, and cancellation hook for file-info
updates. Keep `ColumnProvider`, `MenuProvider`, and `PropertiesModelProvider`
callbacks cheap and synchronous; if they need expensive metadata, read it from a
cache filled by `InfoProvider` or downstream worker code.

The crate does not impose Tokio, async-std, or another runtime. Use a bounded
worker queue or your chosen runtime for blocking work, then call
`UpdateCompletion::complete_with` to apply results back on Nautilus' main
context.

## Requirements

Install the Nautilus extension development package for your distribution:

- Debian/Ubuntu: `libnautilus-extension-dev`
- Fedora: `nautilus-devel`
- Arch: `libnautilus-extension`

The build probes `libnautilus-extension-4` first and then
`libnautilus-extension`, requiring Nautilus 43+ pkg-config metadata.

The crate builds with Rust 1.92 or newer.

Without that package the crate still compiles, which is what docs.rs and
Nautilus-less CI images use, but every Nautilus call is an inert stub. Check
`NATIVE_API_AVAILABLE` if your own code needs to tell the two apart.

## Example

```rust
use nautilus_extension::{
    Column, ColumnProvider, FileInfo, InfoProvider, NautilusModule, OperationResult,
    UpdateFileInfoOperation,
};
use nautilus_extension::gobject_ffi::GTypeModule;

const ATTRIBUTE: &str = "example_attribute";

struct Provider;

impl ColumnProvider for Provider {
    fn get_columns(&self) -> Vec<Column> {
        vec![Column::new(
            "Example::metadata",
            ATTRIBUTE,
            "Example",
            "Example metadata",
        )
        .visible(true)]
    }
}

impl InfoProvider for Provider {
    fn update_file_info_full(&self, operation: UpdateFileInfoOperation) -> OperationResult {
        operation
            .file_info()
            .add_string_attribute(ATTRIBUTE, "example value");
        OperationResult::Complete
    }
}

fn register(module: *mut GTypeModule) -> nautilus_extension::glib_ffi::GType {
    let mut module = NautilusModule::new(module, "RustExampleExtension");
    module
        .add_column_provider(Provider)
        .add_info_provider(Provider)
        .register()
}

nautilus_extension::nautilus_module!(register);
```

## Examples

This package includes neutral example extension modules:

```sh
cargo build --example column_provider
cargo build --example menu_provider
cargo build --example properties_model_provider
```

Install built `.so` files into the Nautilus API 4 `extensions-4` directory and
restart Nautilus with `nautilus -q`.

## Documentation

Full API documentation is published on docs.rs:

- <https://docs.rs/nautilus-extension>
- <https://docs.rs/nautilus-extension-sys>

Users upgrading from the Nautilus 3 crate can start with the
[migration guide](https://github.com/talklittle/nautilus-extension-rs/blob/master/docs/NAUTILUS_3_MIGRATION.md).
