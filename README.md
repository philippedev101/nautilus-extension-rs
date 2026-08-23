# nautilus-extension-rs

Rust bindings for the modern Nautilus extension API used by Nautilus 43 and
newer.

This crate targets the Nautilus API 4 ABI:

* pkg-config: `libnautilus-extension-4` on many distributions; current
  Nautilus documentation also lists `libnautilus-extension`
* shared library: `libnautilus-extension.so.4`
* GObject introspection namespace: `Nautilus-4.0` or `Nautilus-4.1`
* extension install directory: `nautilus/extensions-4`

Nautilus itself is still the GNOME Files application. The "4" here refers to
the extension API and ABI introduced with the Nautilus 43 GTK 4 port.

## API Surface

The safe crate exposes wrappers for the current provider interfaces:

* `ColumnProvider`
* `InfoProvider`, including async `update_file_info_full`, `PendingUpdate`,
  `OperationHandle`, `UpdateCompletion`, `UpdateCompleteCallback`, and
  `cancel_update`
* `MenuProvider`, including `MenuProviderHandle::emit_items_updated_signal`
  and `connect_items_updated`
* `PropertiesModelProvider`
* Advanced `FileInfoImpl` registration for Rust-backed objects that need to
  implement Nautilus' `FileInfo` interface directly

It also includes Rust builders and owned object wrappers for the current API 4
objects:

* `Column` and `ColumnObject`, including `attribute_q`, visibility, alignment,
  and default sort-order accessors
* `FileInfo` and `FileInfoList`, including lookup/create helpers, string
  attributes, emblems, parent/location/mount helpers, list copy/free ownership,
  and `invalidate_extension_info`
* `Menu`, `MenuObject`, `MenuItem`, `MenuItemObject`, `MenuItemType`, and
  `MenuItemList`, including submenu attachment, item activation, and
  sensitivity. Nautilus API 4 does not expose a documented submenu-clearing
  call; Rust helpers report that explicitly instead of faking one.
  Deprecated Nautilus 4.1 menu properties such as `tip`, `icon`, and `priority`
  remain available as compatibility-only Rust APIs.
  `MenuItemObject::connect_activate` exposes the raw GObject activation signal
  when the builder callback is not the right fit. Advanced extensions can also
  register a `NautilusMenuItem` subtype with `MenuItemActivate` through
  `NautilusModule::try_register_menu_item_type` to override the documented
  `MenuItemClass.activate` virtual method.
* `PropertiesModel`, `PropertiesModelObject`, `PropertiesItem`, and
  `PropertiesItemObject`, including access to the underlying `GListModel`
* `OperationResult`, `OperationHandle`, and provider instance handles for
  advanced direct calls into Nautilus provider interfaces
* `type_()` accessors on the object/provider handle wrappers for the documented
  Nautilus `*_get_type()` functions
* `NATIVE_API_AVAILABLE`, which reports whether this build links against the
  native Nautilus library

The `nautilus_module!` macro exports the required C ABI symbols:
`nautilus_module_initialize`, `nautilus_module_list_types`, and
`nautilus_module_shutdown`. Registration functions may return one `GType`, an
`Option<GType>`, a `Vec<GType>`, or a fixed-size array of `GType` values.

The obsolete Nautilus 3 APIs are intentionally not part of the primary API:

* no GTK 3 dependency
* no `LocationWidgetProvider`
* no GTK-widget `PropertyPageProvider`
* no `GtkWidget`/window argument in menu provider callbacks

Version `0.8.0` of this crate targeted Nautilus 3. Code using the old property
page or menu signatures needs source changes for Nautilus 4.

## Requirements

Install the Nautilus extension development package for your distribution:

* Debian/Ubuntu: `libnautilus-extension-dev`
* Fedora: `nautilus-devel`
* Arch: `libnautilus-extension`

You also need Rust and the GLib/GObject/Gio development files. The build probes
`libnautilus-extension-4` first and then `libnautilus-extension`, with a
Nautilus 43+ version floor for both names so old Nautilus 3 development files do
not satisfy the API 4 build.

## Building Examples

The crate includes three cdylib examples:

```sh
cargo build --example column_provider
cargo build --example menu_provider
cargo build --example properties_model_provider
```

The output shared objects are under `target/debug/examples/`.

The `menu_provider` example adds menu items for selected files and the current
folder. The `column_provider` example registers a visible `Demo Status` column
and fills it through `InfoProvider` with neutral file metadata. The
`properties_model_provider` example adds a small model-based properties page
with generic rows.

## Testing

Run the unit tests normally on a system with the Nautilus API 4 development
files installed:

```sh
cargo test --all-targets
```

For CI or development environments that have GLib but not Nautilus installed,
the explicit bypass can run unit tests without linking
`libnautilus-extension.so.4`. In this mode, safe wrappers that would normally
call Nautilus return neutral defaults such as `None`, `false`, or
`OperationResult::Failed`:

```sh
NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG=1 cargo test --all-targets
```

That bypass is not suitable for building a loadable Nautilus extension.

The wrapper source stays single-path across both builds: the stub definitions
live next to the `extern "C"` declarations in `nautilus-extension-sys`, and the
public `NATIVE_API_AVAILABLE` constant reports which build this is for the few
places whose result would otherwise be misleading, such as GObject type
registration.

Maintainers should run the fast validation suite before commits that change API
surface, FFI behavior, or project structure:

```sh
bash scripts/validation/all-fast.sh
```

`.github/workflows/validation.yml` runs on every push and pull request:

* **Fast validation** runs `all-fast.sh` without Nautilus installed, so it also
  covers the unlinked build.
* **Native Nautilus validation** runs `native-strong.sh` against a real Nautilus
  on Ubuntu, Fedora and Arch, which is what catches API drift between distros.
* **Minimum supported Rust version** compiles against the declared
  `rust-version` with the committed lock file.
* **Package validation** and **Semver policy** run the publish dry-run and the
  API comparison.

Running `all-fast.sh` locally is still worth it, because it is the fastest way
to find a problem, and because CI cannot check the scripts it is itself running.

Additional project maintenance docs:

* [Nautilus 3 migration guide](docs/NAUTILUS_3_MIGRATION.md)
* [Nautilus API 4 coverage tracking](docs/API_TRACKING.md)
* [Public API naming audit](docs/API_NAMING.md)
* [Architecture and project structure](docs/ARCHITECTURE.md)
* [FFI panic and unsafe audit](docs/FFI_AUDIT.md)
* [Versioning policy](docs/VERSIONING.md)
* [Release checklist](docs/RELEASE.md)

## Installing Extensions

Install the built `.so` into the Nautilus API 4 extension directory, then restart
Nautilus.

Common system paths are:

* Debian/Ubuntu: `/usr/lib/$(gcc -dumpmachine)/nautilus/extensions-4/`
* Fedora: `/usr/lib64/nautilus/extensions-4/`
* Arch: `/usr/lib/nautilus/extensions-4/`

Some distributions expose the exact directory through pkg-config:

```sh
pkg-config --variable=extensiondir libnautilus-extension-4
pkg-config --variable=extensiondir libnautilus-extension
```

After copying an extension, restart Nautilus:

```sh
nautilus -q
```

## Writing a Column Extension

A column extension normally implements both `ColumnProvider` and `InfoProvider`.
The column's `attribute` must match the string attribute set on each
`FileInfo`.

```rust
use nautilus_extension::{
    Column, ColumnProvider, FileInfo, InfoProvider, OperationResult,
    UpdateFileInfoOperation,
};

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
```

For expensive work, return `OperationResult::InProgress`, move the
`PendingUpdate` completion and handle into a bounded worker queue, and call
`completion.complete_with(...)` when the attribute is ready. `FileInfo` access
is marshaled back to Nautilus' main context by `complete_with`; do not move
`FileInfo` itself into worker threads. Avoid spawning one OS thread per file in
production, because Nautilus may request information for many files in one
directory view.

```rust
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use nautilus_extension::{
    InfoProvider, OperationHandle, OperationResult, UpdateCompletion,
    UpdateFileInfoOperation,
};

const ATTRIBUTE: &str = "example_attribute";

struct WorkItem {
    name: String,
    handle: OperationHandle,
    completion: UpdateCompletion,
}

struct Provider {
    sender: SyncSender<WorkItem>,
}

impl Provider {
    fn new() -> Provider {
        let (sender, receiver) = sync_channel::<WorkItem>(32);

        std::thread::spawn(move || {
            while let Ok(work) = receiver.recv() {
                if work.handle.is_cancelled() {
                    let _ = work.completion.complete(OperationResult::Failed);
                    continue;
                }

                let value = format!("name length: {}", work.name.len());
                let _ = work.completion.complete_with(OperationResult::Complete, move |file| {
                    file.add_string_attribute(ATTRIBUTE, &value);
                });
            }
        });

        Provider { sender }
    }
}

impl InfoProvider for Provider {
    fn update_file_info_full(&self, operation: UpdateFileInfoOperation) -> OperationResult {
        let name = operation.file_info().name().unwrap_or_default();
        let pending = operation.into_pending();
        let item = WorkItem {
            name,
            handle: pending.handle().clone(),
            completion: pending.completion(),
        };

        match self.sender.try_send(item) {
            Ok(()) => OperationResult::InProgress,
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                OperationResult::Failed
            }
        }
    }
}
```

## Async Model

Nautilus API 4 has a real asynchronous completion contract only for
`InfoProvider`. Use `update_file_info_full` when work may block or take
noticeable time, return `OperationResult::InProgress`, check
`OperationHandle::is_cancelled` from the worker side, and finish exactly once
with `UpdateCompletion::complete` or `UpdateCompletion::complete_with`.

Other provider traits stay synchronous because Nautilus asks for their results
immediately. `ColumnProvider` should declare columns cheaply, `MenuProvider`
should build context-menu items from already-available state, and
`PropertiesModelProvider` should return model rows without starting long work.
If those providers need expensive data, compute it ahead of time through an
`InfoProvider` or a downstream cache. For changing menu state, update the cache
and emit `MenuProviderHandle::emit_items_updated_signal` rather than trying to
construct menu items asynchronously.

This crate does not require Tokio, async-std, or another general Rust async
runtime. Downstream extensions can choose their own runtime or worker-pool
strategy; the wrapper only provides the Nautilus and GLib main-context handoff
needed to complete file-info updates safely.

## References

* Nautilus API documentation: <https://gnome.pages.gitlab.gnome.org/nautilus/>
* Nautilus Python API 4 migration notes:
  <https://gnome.pages.gitlab.gnome.org/nautilus-python/nautilus-python-migrating-to-4.html>
* GNOME GTK 4 port warning:
  <https://discourse.gnome.org/t/reminder-for-projects-shipping-a-nautilus-extension-you-must-port-to-gtk4/10462>

## Release notes

See the [changelog](CHANGELOG.md) for changes between versions.

## License

[GNU General Public License version 3][gpl-3]

[gpl-3]: COPYING.txt
