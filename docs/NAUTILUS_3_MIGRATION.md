# Migrating from Nautilus 3 Bindings

The original `nautilus-extension` crate targeted the Nautilus 3 / GTK 3
extension API. This branch targets Nautilus API 4, introduced with Nautilus 43
and the GTK 4 port.

Nautilus itself did not become "Nautilus 4"; the extension API and ABI did.

## Build and Install Changes

| Nautilus 3 era | Nautilus API 4 era |
| --- | --- |
| GTK 3 extension ABI | GTK 4-era extension ABI |
| old `libnautilus-extension` development files | `libnautilus-extension-4` or current `libnautilus-extension` pkg-config data |
| `libnautilus-extension.so` older ABI | `libnautilus-extension.so.4` |
| `nautilus/extensions-3.0` style directories | `nautilus/extensions-4` |

The build script requires Nautilus 43+ pkg-config metadata, so old Nautilus 3
development packages should not satisfy this crate.

## Removed or Replaced APIs

Nautilus API 4 removed direct GTK widget manipulation from extension modules.
This crate does not reintroduce those obsolete GTK 3 APIs.

| Nautilus 3 API | Nautilus API 4 replacement |
| --- | --- |
| `LocationWidgetProvider` | removed upstream; no replacement in this crate |
| GTK-widget `PropertyPageProvider` | `PropertiesModelProvider` |
| menu provider callbacks with a window argument | `MenuProvider` callbacks without a window argument |
| GTK 3 dependency in extension crates | no GTK 3 dependency |

## Provider Migration Notes

`ColumnProvider` remains the entry point for declaring custom columns. Combine
it with `InfoProvider` to fill the matching string attribute for each file.

`InfoProvider` supports both synchronous `update_file_info` and asynchronous
`update_file_info_full`. Expensive work should return
`OperationResult::InProgress`, keep the `OperationHandle` for cancellation
checks, and complete through `UpdateCompletion`.

`MenuProvider` returns `MenuItem` values for file selections and background
menus. Nautilus API 4 no longer passes a GTK window to these callbacks.

`PropertiesModelProvider` returns model-backed properties sections. It is not a
GTK widget API.

## Naming Notes

Provider trait method names intentionally mirror the Nautilus C interface names
such as `get_columns`, `get_file_items`, and `get_models`. Wrapper handles also
provide Rust-style aliases such as `columns`, `file_items`, `background_items`,
and `models` where that improves call-site readability.
