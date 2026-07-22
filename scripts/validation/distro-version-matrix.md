# Cross-Distro and Nautilus-Version Validation

The fast and native validation scripts are local and deterministic. Distro and
version coverage is intentionally separated because it requires containers or
VMs with different Nautilus development packages.

Recommended package mapping:

- Debian/Ubuntu: `libnautilus-extension-dev`, `pkg-config`, `gcc`
- Fedora: `nautilus-devel`, `pkgconf-pkg-config`, `gcc`
- Arch: `nautilus`, `pkgconf`, `gcc`
- Alpine: `nautilus-dev`, `pkgconf`, `gcc`, `musl-dev`
- NixOS/Nix: `nautilus`, `pkg-config`, `gcc`

Each environment should run:

```sh
bash scripts/validation/native-strong.sh
```

For older API 4.0 environments, the API surface checker should be run with the
installed `Nautilus-4.0.gir`. For newer API 4.1+ environments, it should fail
if the GIR exposes additional public classes, interfaces, records, enums, or
functions that are not represented in `scripts/validation/check-api-surface.pl`.
