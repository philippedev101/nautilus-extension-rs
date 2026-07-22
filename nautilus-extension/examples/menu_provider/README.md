# Menu Provider Example

This example demonstrates neutral Nautilus context-menu items.

It adds one item for selected files and one item for the current folder
background. Activating either item prints known URIs to stderr; it does not run
external commands or modify files.

Build:

```sh
cargo build --example menu_provider
```

Install the produced shared object into the Nautilus API 4 `extensions-4`
directory:

```sh
install -Dm755 target/debug/examples/libmenu_provider.so \
  "$(pkg-config --variable=extensiondir libnautilus-extension-4)/libmenu_provider.so"
nautilus -q
```

On distributions that expose only the unversioned pkg-config file for the API 4
library, replace `libnautilus-extension-4` with `libnautilus-extension`.
