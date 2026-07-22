# Column Provider Example

This example demonstrates a custom Nautilus list-view column backed by an
`InfoProvider`.

It declares a `Demo Status` column whose attribute is `demo_status`, then fills
that attribute with neutral metadata already known by Nautilus: URI scheme and
MIME type.

Build:

```sh
cargo build --example column_provider
```

Install the produced shared object into the Nautilus API 4 `extensions-4`
directory:

```sh
install -Dm755 target/debug/examples/libcolumn_provider.so \
  "$(pkg-config --variable=extensiondir libnautilus-extension-4)/libcolumn_provider.so"
nautilus -q
```

On distributions that expose only the unversioned pkg-config file for the API 4
library, replace `libnautilus-extension-4` with `libnautilus-extension`.
