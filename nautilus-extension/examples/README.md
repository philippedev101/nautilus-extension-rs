# Examples

The examples build as Nautilus API 4 extension shared objects. They are neutral
demonstrations of the reusable library API and do not implement app-specific
metadata extraction.

Build all examples:

```sh
cargo build -p nautilus-extension --examples
```

The debug shared objects are written to `target/debug/examples/`.

Common Nautilus API 4 extension directories:

- Debian/Ubuntu: `/usr/lib/$(gcc -dumpmachine)/nautilus/extensions-4/`
- Fedora: `/usr/lib64/nautilus/extensions-4/`
- Arch: `/usr/lib/nautilus/extensions-4/`

If pkg-config exposes the extension directory, prefer it:

```sh
pkg-config --variable=extensiondir libnautilus-extension-4
pkg-config --variable=extensiondir libnautilus-extension
```

Restart Nautilus after installing an example:

```sh
nautilus -q
```

Example-specific notes:

- `column_provider`: declares a `Demo Status` column and fills it through
  `InfoProvider` with URI scheme and MIME type data.
- `menu_provider`: adds neutral context-menu entries and prints URIs to stderr.
- `properties_model_provider`: adds a model-backed `Rust Demo` properties
  section with generic file rows.
