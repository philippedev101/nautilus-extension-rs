# Public API Naming Audit

This audit applies to the Nautilus API 4 Rust wrapper surface.

The Nautilus extension ABI is a GObject C API, so several provider method names
intentionally mirror upstream names:

- `ColumnProvider::get_columns`
- `MenuProvider::get_file_items`
- `MenuProvider::get_background_items`
- `PropertiesModelProvider::get_models`

Keeping these trait method names makes it clear which Nautilus interface method
an implementation satisfies and reduces confusion when cross-referencing the C
documentation.

Wrapper objects should prefer Rust-style accessors when they are not directly
implementing a Nautilus interface. Compatibility methods may stay available, but
new wrapper APIs should avoid unnecessary `get_` prefixes.

## Current Rust-Style Aliases

| Compatibility method | Preferred alias |
| --- | --- |
| `ColumnProviderHandle::get_columns` | `ColumnProviderHandle::columns` |
| `MenuProviderHandle::get_file_items` | `MenuProviderHandle::file_items` |
| `MenuProviderHandle::get_background_items` | `MenuProviderHandle::background_items` |
| `MenuObject::get_items` | `MenuObject::items` |
| `PropertiesModelProviderHandle::get_models` | `PropertiesModelProviderHandle::models` |
| `FileInfo::get_uri` | `FileInfo::uri_or_empty` |
| `FileInfo::get_uri_scheme` | `FileInfo::uri_scheme_or_empty` |

## Conversion Naming

Use these conventions for new wrapper methods:

- `as_ptr` for borrowed raw pointers.
- `raw` for compatibility with the existing wrapper API.
- `into_raw` when ownership is transferred out of the Rust wrapper.
- `from_raw_borrowed` when a wrapper adds a reference.
- `from_raw_full` when a wrapper consumes a full-transfer reference.

Do not remove compatibility aliases in a patch release. If a compatibility name
needs to be deprecated or removed, document it in `CHANGELOG.md` and bump the
minor version while the crates remain pre-1.0.
