# Nautilus API 4 Coverage Tracking

This document is the human-readable coverage map for Nautilus API 4. Keep it in
sync with `scripts/validation/check-api-surface.pl` whenever the C headers or
GIR metadata expose new API.

## Classes

| Nautilus API item | sys coverage | safe wrapper |
| --- | --- | --- |
| `Column` | `NautilusColumnClass`, `nautilus_column_get_type`, `nautilus_column_new` | `Column`, `ColumnObject`, `ColumnSortOrder` |
| `Menu` | `NautilusMenuClass`, `nautilus_menu_get_type`, `nautilus_menu_new`, `nautilus_menu_append_item`, `nautilus_menu_get_items` | `Menu`, `MenuObject` |
| `MenuItem` | `NautilusMenuItemClass`, `nautilus_menu_item_get_type`, `nautilus_menu_item_new`, `nautilus_menu_item_activate`, `nautilus_menu_item_set_submenu`, `nautilus_menu_item_list_free` | `MenuItem`, `MenuItemObject`, `MenuItemList`, `MenuItemType`, `MenuItemActivate`, `MenuActivation`, `MenuActivationTarget` |
| `PropertiesItem` | `NautilusPropertiesItemClass`, `nautilus_properties_item_get_type`, `nautilus_properties_item_new`, `nautilus_properties_item_get_name`, `nautilus_properties_item_get_value` | `PropertiesItem`, `PropertiesItemObject` |
| `PropertiesModel` | `NautilusPropertiesModelClass`, `nautilus_properties_model_get_type`, `nautilus_properties_model_new`, `nautilus_properties_model_get_model`, `nautilus_properties_model_get_title`, `nautilus_properties_model_set_title` | `PropertiesModel`, `PropertiesModelObject` |

## Interfaces

| Nautilus API item | sys coverage | safe wrapper |
| --- | --- | --- |
| `ColumnProvider` | `NautilusColumnProviderIface`, `nautilus_column_provider_get_type`, `nautilus_column_provider_get_columns` | `ColumnProvider`, `ColumnProviderHandle` |
| `FileInfo` | `NautilusFileInfoInterface`, `nautilus_file_info_get_type`, `nautilus_file_info_create`, `nautilus_file_info_create_for_uri`, `nautilus_file_info_lookup`, `nautilus_file_info_lookup_for_uri`, `nautilus_file_info_list_copy`, `nautilus_file_info_list_free` | `FileInfo`, `FileInfoHandle`, `FileInfoImpl`, `FileInfoList`, `OwnedGObject` |
| `InfoProvider` | `NautilusInfoProviderIface`, `nautilus_info_provider_get_type`, `nautilus_info_provider_update_file_info`, `nautilus_info_provider_cancel_update`, `nautilus_info_provider_update_complete_invoke` | `InfoProvider`, `InfoProviderHandle`, `PendingUpdate`, `UpdateFileInfoOperation`, `UpdateCompleteCallback`, `UpdateCompletion` |
| `MenuProvider` | `NautilusMenuProviderIface`, `nautilus_menu_provider_get_type`, `nautilus_menu_provider_get_file_items`, `nautilus_menu_provider_get_background_items`, `nautilus_menu_provider_emit_items_updated_signal` | `MenuProvider`, `MenuProviderHandle`, `SignalHandlerId` |
| `PropertiesModelProvider` | `NautilusPropertiesModelProviderIface`, `nautilus_properties_model_provider_get_type`, `nautilus_properties_model_provider_get_models` | `PropertiesModelProvider`, `PropertiesModelProviderHandle` |

## FileInfo Methods

| Nautilus API item | safe wrapper |
| --- | --- |
| `nautilus_file_info_add_emblem` | `FileInfo::add_emblem`, `FileInfoImpl::add_emblem` |
| `nautilus_file_info_add_string_attribute` | `FileInfo::add_string_attribute`, `FileInfo::add_attribute`, `FileInfoImpl::add_string_attribute` |
| `nautilus_file_info_can_write` | `FileInfo::can_write`, `FileInfoImpl::can_write` |
| `nautilus_file_info_get_activation_uri` | `FileInfo::activation_uri`, `FileInfoImpl::activation_uri` |
| `nautilus_file_info_get_file_type` | `FileInfo::file_type`, `FileInfoImpl::file_type` |
| `nautilus_file_info_get_location` | `FileInfo::location`, `FileInfo::location_uri`, `FileInfo::location_path`, `FileInfoImpl::location` |
| `nautilus_file_info_get_mime_type` | `FileInfo::mime_type`, `FileInfoImpl::mime_type` |
| `nautilus_file_info_get_mount` | `FileInfo::mount`, `FileInfoImpl::mount` |
| `nautilus_file_info_get_name` | `FileInfo::name`, `FileInfoImpl::name` |
| `nautilus_file_info_get_parent_info` | `FileInfo::parent_info`, `FileInfoImpl::parent_info` |
| `nautilus_file_info_get_parent_location` | `FileInfo::parent_location`, `FileInfoImpl::parent_location` |
| `nautilus_file_info_get_parent_uri` | `FileInfo::parent_uri`, `FileInfoImpl::parent_uri` |
| `nautilus_file_info_get_string_attribute` | `FileInfo::string_attribute`, `FileInfoImpl::string_attribute` |
| `nautilus_file_info_get_uri` | `FileInfo::uri`, `FileInfo::get_uri`, `FileInfo::uri_or_empty`, `FileInfoImpl::uri` |
| `nautilus_file_info_get_uri_scheme` | `FileInfo::uri_scheme`, `FileInfo::get_uri_scheme`, `FileInfo::uri_scheme_or_empty`, `FileInfoImpl::uri_scheme` |
| `nautilus_file_info_invalidate_extension_info` | `FileInfo::invalidate_extension_info`, `FileInfoImpl::invalidate_extension_info` |
| `nautilus_file_info_is_directory` | `FileInfo::is_directory`, `FileInfoImpl::is_directory` |
| `nautilus_file_info_is_gone` | `FileInfo::is_gone`, `FileInfoImpl::is_gone` |
| `nautilus_file_info_is_mime_type` | `FileInfo::is_mime_type`, `FileInfoImpl::is_mime_type` |

## Records, Enums, and Handles

| Nautilus API item | sys coverage | safe wrapper |
| --- | --- | --- |
| `OperationResult` | `NautilusOperationResult`, `nautilus_operation_result_get_type` | `OperationResult` |
| `OperationHandle` | `NautilusOperationHandle` | `OperationHandle` |
| `ColumnProviderInterface` | `NautilusColumnProviderIface` | generated provider trampoline slots |
| `FileInfoInterface` | `NautilusFileInfoInterface` | generated `FileInfoImpl` trampoline slots |
| `InfoProviderInterface` | `NautilusInfoProviderIface` | generated provider trampoline slots |
| `MenuProviderInterface` | `NautilusMenuProviderIface` | generated provider trampoline slots |
| `PropertiesModelProviderInterface` | `NautilusPropertiesModelProviderIface` | generated provider trampoline slots |

## Module Entry Points

| C ABI symbol | Rust support |
| --- | --- |
| `nautilus_module_initialize` | exported by `nautilus_module!` |
| `nautilus_module_list_types` | exported by `nautilus_module!` |
| `nautilus_module_shutdown` | exported by `nautilus_module!` |
| `module_initialize` | GIR-level function represented by `nautilus_module!` |
| `module_list_types` | GIR-level function represented by `nautilus_module!` |
| `module_shutdown` | GIR-level function represented by `nautilus_module!` |

## Registration and Helper Types

The safe registration layer exposes:

- `NautilusModule`
- `NautilusModuleError`
- `IntoModuleTypes`
- `nautilus_module`

The generated trampoline slots currently cover:

- `MAX_COLUMN_PROVIDERS`
- `MAX_FILE_INFO_IMPLS`
- `MAX_INFO_PROVIDERS`
- `MAX_MENU_ITEM_ACTIVATORS`
- `MAX_MENU_PROVIDERS`
- `MAX_PROPERTIES_MODEL_PROVIDERS`

## Known Non-Goals

Nautilus 3-only APIs are not covered by this Nautilus API 4 binding layer:

- GTK-widget `PropertyPageProvider`
- `LocationWidgetProvider`
- menu callbacks with GTK window arguments
- GTK 3 dependencies
