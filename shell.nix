{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  packages = with pkgs; [
    binutils
    cargo-fuzz
    cargo-llvm-cov
    cargo-semver-checks
    dbus
    gcc
    glib
    gobject-introspection
    nautilus
    pkg-config
    xorg-server
    valgrind
  ];

  shellHook = ''
    export NAUTILUS_EXTENSION_RS_USE_SYSTEM_NAUTILUS4=1
    echo "Nautilus extension dev shell"
    echo "  pkg-config: $(pkg-config --modversion libnautilus-extension-4 2>/dev/null || true)"
    echo "  extensiondir: $(pkg-config --variable=extensiondir libnautilus-extension-4 2>/dev/null || true)"
  '';
}
