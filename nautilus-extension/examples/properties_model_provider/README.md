# Properties Model Provider Example

This example demonstrates Nautilus API 4 model-backed properties sections.

It adds a `Rust Demo` section for a single selected file, with rows for name
length, URI scheme, and MIME type. It does not create GTK widgets directly.

Build:

```sh
cargo build --example properties_model_provider
```

Install the produced shared object into the Nautilus API 4 `extensions-4`
directory:

```sh
install -Dm755 target/debug/examples/libproperties_model_provider.so \
  "$(pkg-config --variable=extensiondir libnautilus-extension-4)/libproperties_model_provider.so"
nautilus -q
```

On distributions that expose only the unversioned pkg-config file for the API 4
library, replace `libnautilus-extension-4` with `libnautilus-extension`.
