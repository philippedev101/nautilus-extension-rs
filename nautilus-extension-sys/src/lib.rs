#![deny(bare_trait_objects)]
#![allow(non_camel_case_types)]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(unsafe_op_in_unsafe_fn)]
//! Low-level FFI bindings for Nautilus API 4.
//!
//! This crate exposes the C ABI from `libnautilus-extension.so.4` with raw
//! pointers and C ownership rules. Most extension authors should use the safe
//! wrappers in the `nautilus-extension` crate instead.

mod unlinked;

use crate::unlinked::nautilus_api;

/// Whether this build links against the native Nautilus extension library.
///
/// This is `false` only when the library was unavailable at build time, such
/// as on docs.rs. Every function below is then a stub that returns a neutral
/// value, so callers that must behave differently should branch on this
/// constant at run time.
pub const NATIVE_API_AVAILABLE: bool = cfg!(not(nautilus_extension_rs_skip_link));

extern crate gio_sys as gio_ffi;
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
extern crate libc;

use crate::gio_ffi::{GFile, GFileType, GListModel, GMount};
use crate::glib_ffi::{gboolean, GList, GType};
use crate::gobject_ffi::{GClosure, GObjectClass, GTypeInterface};
use libc::c_char;

#[repr(C)]
/// Class struct for `NautilusColumn`.
pub struct NautilusColumnClass {
    /// Parent GObject class.
    pub parent_class: GObjectClass,
}

#[repr(C)]
/// Class struct for `NautilusMenu`.
pub struct NautilusMenuClass {
    /// Parent GObject class.
    pub parent_class: GObjectClass,
}

#[repr(C)]
/// Class struct for `NautilusMenuItem`.
pub struct NautilusMenuItemClass {
    /// Parent GObject class.
    pub parent: GObjectClass,
    /// Virtual method invoked when the item is activated.
    pub activate: Option<unsafe extern "C" fn(*mut NautilusMenuItem)>,
}

#[repr(C)]
/// Class struct for `NautilusPropertiesItem`.
pub struct NautilusPropertiesItemClass {
    /// Parent GObject class.
    pub parent_class: GObjectClass,
}

#[repr(C)]
/// Class struct for `NautilusPropertiesModel`.
pub struct NautilusPropertiesModelClass {
    /// Parent GObject class.
    pub parent_class: GObjectClass,
}

#[repr(C)]
/// Interface vtable for `NautilusFileInfo`.
pub struct NautilusFileInfoInterface {
    /// Parent GType interface data.
    pub g_iface: GTypeInterface,
    /// Returns whether the file is gone.
    pub is_gone: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> gboolean>,
    /// Returns the file display name.
    pub get_name: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut c_char>,
    /// Returns the file URI.
    pub get_uri: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut c_char>,
    /// Returns the parent URI.
    pub get_parent_uri: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut c_char>,
    /// Returns the URI scheme.
    pub get_uri_scheme: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut c_char>,
    /// Returns the MIME type.
    pub get_mime_type: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut c_char>,
    /// Returns whether the file matches a MIME type.
    pub is_mime_type:
        Option<unsafe extern "C" fn(*mut NautilusFileInfo, *const c_char) -> gboolean>,
    /// Returns whether the file is a directory.
    pub is_directory: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> gboolean>,
    /// Adds an emblem by icon name.
    pub add_emblem: Option<unsafe extern "C" fn(*mut NautilusFileInfo, *const c_char)>,
    /// Returns a string attribute.
    pub get_string_attribute:
        Option<unsafe extern "C" fn(*mut NautilusFileInfo, *const c_char) -> *mut c_char>,
    /// Adds or updates a string attribute.
    pub add_string_attribute:
        Option<unsafe extern "C" fn(*mut NautilusFileInfo, *const c_char, *const c_char)>,
    /// Invalidates extension-provided information.
    pub invalidate_extension_info: Option<unsafe extern "C" fn(*mut NautilusFileInfo)>,
    /// Returns the activation URI.
    pub get_activation_uri: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut c_char>,
    /// Returns the Gio file type.
    pub get_file_type: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> GFileType>,
    /// Returns the Gio location.
    pub get_location: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut GFile>,
    /// Returns the parent Gio location.
    pub get_parent_location: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut GFile>,
    /// Returns the parent file info.
    pub get_parent_info:
        Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut NautilusFileInfo>,
    /// Returns the containing mount.
    pub get_mount: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> *mut GMount>,
    /// Returns whether the file is writable.
    pub can_write: Option<unsafe extern "C" fn(*mut NautilusFileInfo) -> gboolean>,
}

#[repr(C)]
/// Interface vtable for `NautilusColumnProvider`.
pub struct NautilusColumnProviderIface {
    /// Parent GType interface data.
    pub g_iface: GTypeInterface,
    /// Returns provided columns.
    pub get_columns: Option<unsafe extern "C" fn(*mut NautilusColumnProvider) -> *mut GList>,
}

/// Compatibility alias for `NautilusColumnProviderIface`.
pub type NautilusColumnProviderInterface = NautilusColumnProviderIface;

#[repr(C)]
/// Interface vtable for `NautilusInfoProvider`.
pub struct NautilusInfoProviderIface {
    /// Parent GType interface data.
    pub g_iface: GTypeInterface,
    /// Updates file information.
    pub update_file_info: Option<
        unsafe extern "C" fn(
            *mut NautilusInfoProvider,
            *mut NautilusFileInfo,
            *mut GClosure,
            *mut *mut NautilusOperationHandle,
        ) -> NautilusOperationResult,
    >,
    /// Cancels an in-progress update.
    pub cancel_update:
        Option<unsafe extern "C" fn(*mut NautilusInfoProvider, *mut NautilusOperationHandle)>,
}

/// Compatibility alias for `NautilusInfoProviderIface`.
pub type NautilusInfoProviderInterface = NautilusInfoProviderIface;

#[repr(C)]
/// Interface vtable for `NautilusMenuProvider`.
pub struct NautilusMenuProviderIface {
    /// Parent GType interface data.
    pub g_iface: GTypeInterface,
    /// Returns menu items for selected files.
    pub get_file_items:
        Option<unsafe extern "C" fn(*mut NautilusMenuProvider, *mut GList) -> *mut GList>,
    /// Returns menu items for the current folder background.
    pub get_background_items: Option<
        unsafe extern "C" fn(*mut NautilusMenuProvider, *mut NautilusFileInfo) -> *mut GList,
    >,
}

/// Compatibility alias for `NautilusMenuProviderIface`.
pub type NautilusMenuProviderInterface = NautilusMenuProviderIface;

#[repr(C)]
/// Interface vtable for `NautilusPropertiesModelProvider`.
pub struct NautilusPropertiesModelProviderIface {
    /// Parent GType interface data.
    pub g_iface: GTypeInterface,
    /// Returns properties models for selected files.
    pub get_models: Option<
        unsafe extern "C" fn(*mut NautilusPropertiesModelProvider, *mut GList) -> *mut GList,
    >,
}

/// Compatibility alias for `NautilusPropertiesModelProviderIface`.
pub type NautilusPropertiesModelProviderInterface = NautilusPropertiesModelProviderIface;

/// Opaque `NautilusColumn` type.
pub enum NautilusColumn {}
/// Opaque `NautilusColumnProvider` type.
pub enum NautilusColumnProvider {}
/// Opaque `NautilusFileInfo` type.
pub enum NautilusFileInfo {}
/// Opaque `NautilusInfoProvider` type.
pub enum NautilusInfoProvider {}
/// Opaque `NautilusMenu` type.
pub enum NautilusMenu {}
/// Opaque `NautilusMenuItem` type.
pub enum NautilusMenuItem {}
/// Opaque `NautilusMenuProvider` type.
pub enum NautilusMenuProvider {}
/// Opaque `NautilusOperationHandle` type.
pub enum NautilusOperationHandle {}
/// Opaque `NautilusPropertiesItem` type.
pub enum NautilusPropertiesItem {}
/// Opaque `NautilusPropertiesModel` type.
pub enum NautilusPropertiesModel {}
/// Opaque `NautilusPropertiesModelProvider` type.
pub enum NautilusPropertiesModelProvider {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// C enum returned by Nautilus extension operations.
pub enum NautilusOperationResult {
    /// Operation completed successfully.
    NautilusOperationComplete = 0,
    /// Operation failed.
    NautilusOperationFailed = 1,
    /// Operation will complete asynchronously.
    NautilusOperationInProgress = 2,
}

nautilus_api! {
    /// Returns the GType for `NautilusOperationResult`.
    fn nautilus_operation_result_get_type() -> GType;

    /// Returns the GType for `NautilusColumn`.
    fn nautilus_column_get_type() -> GType;
    /// Creates a new `NautilusColumn`.
    fn nautilus_column_new(
        name: *const c_char,
        attribute: *const c_char,
        label: *const c_char,
        description: *const c_char,
    ) -> *mut NautilusColumn;
    /// Returns the GType for `NautilusColumnProvider`.
    fn nautilus_column_provider_get_type() -> GType;
    /// Calls a column provider's `get_columns` interface method.
    fn nautilus_column_provider_get_columns(
        provider: *mut NautilusColumnProvider,
    ) -> *mut GList;

    /// Returns the GType for `NautilusFileInfo`.
    fn nautilus_file_info_get_type() -> GType;
    /// Creates file info for a Gio file location.
    fn nautilus_file_info_create(location: *mut GFile) -> *mut NautilusFileInfo;
    /// Creates file info for a URI.
    fn nautilus_file_info_create_for_uri(uri: *const c_char) -> *mut NautilusFileInfo;
    /// Looks up file info for a Gio file location.
    fn nautilus_file_info_lookup(location: *mut GFile) -> *mut NautilusFileInfo;
    /// Looks up file info for a URI.
    fn nautilus_file_info_lookup_for_uri(uri: *const c_char) -> *mut NautilusFileInfo;
    /// Copies a `GList` of `NautilusFileInfo` objects.
    fn nautilus_file_info_list_copy(files: *mut GList) -> *mut GList;
    /// Frees a `GList` of `NautilusFileInfo` objects.
    fn nautilus_file_info_list_free(files: *mut GList);
    /// Adds an emblem by icon name.
    fn nautilus_file_info_add_emblem(file: *mut NautilusFileInfo, emblem_name: *const c_char);
    /// Adds or updates a string attribute.
    fn nautilus_file_info_add_string_attribute(
        file: *mut NautilusFileInfo,
        attribute_name: *const c_char,
        value: *const c_char,
    );
    /// Returns whether the file is writable.
    fn nautilus_file_info_can_write(file_info: *mut NautilusFileInfo) -> gboolean;
    /// Returns the activation URI.
    fn nautilus_file_info_get_activation_uri(file_info: *mut NautilusFileInfo) -> *mut c_char;
    /// Returns the Gio file type.
    fn nautilus_file_info_get_file_type(file_info: *mut NautilusFileInfo) -> GFileType;
    /// Returns the Gio location.
    fn nautilus_file_info_get_location(file_info: *mut NautilusFileInfo) -> *mut GFile;
    /// Returns the MIME type.
    fn nautilus_file_info_get_mime_type(file_info: *mut NautilusFileInfo) -> *mut c_char;
    /// Returns the containing mount.
    fn nautilus_file_info_get_mount(file_info: *mut NautilusFileInfo) -> *mut GMount;
    /// Returns the file display name.
    fn nautilus_file_info_get_name(file_info: *mut NautilusFileInfo) -> *mut c_char;
    /// Returns the parent file info.
    fn nautilus_file_info_get_parent_info(
        file_info: *mut NautilusFileInfo,
    ) -> *mut NautilusFileInfo;
    /// Returns the parent Gio location.
    fn nautilus_file_info_get_parent_location(file_info: *mut NautilusFileInfo) -> *mut GFile;
    /// Returns the parent URI.
    fn nautilus_file_info_get_parent_uri(file_info: *mut NautilusFileInfo) -> *mut c_char;
    /// Returns a string attribute.
    fn nautilus_file_info_get_string_attribute(
        file_info: *mut NautilusFileInfo,
        attribute_name: *const c_char,
    ) -> *mut c_char;
    /// Returns the file URI.
    fn nautilus_file_info_get_uri(file_info: *mut NautilusFileInfo) -> *mut c_char;
    /// Returns the URI scheme.
    fn nautilus_file_info_get_uri_scheme(file_info: *mut NautilusFileInfo) -> *mut c_char;
    /// Invalidates extension-provided information.
    fn nautilus_file_info_invalidate_extension_info(file: *mut NautilusFileInfo);
    /// Returns whether the file is a directory.
    fn nautilus_file_info_is_directory(file_info: *mut NautilusFileInfo) -> gboolean;
    /// Returns whether the file is gone.
    fn nautilus_file_info_is_gone(file_info: *mut NautilusFileInfo) -> gboolean;
    /// Returns whether the file matches a MIME type.
    fn nautilus_file_info_is_mime_type(
        file_info: *mut NautilusFileInfo,
        mime_type: *const c_char,
    ) -> gboolean;

    /// Returns the GType for `NautilusInfoProvider`.
    fn nautilus_info_provider_get_type() -> GType;
    /// Calls an info provider's update method.
    fn nautilus_info_provider_update_file_info(
        provider: *mut NautilusInfoProvider,
        file: *mut NautilusFileInfo,
        update_complete: *mut GClosure,
        handle: *mut *mut NautilusOperationHandle,
    ) -> NautilusOperationResult;
    /// Calls an info provider's cancel method.
    fn nautilus_info_provider_cancel_update(
        provider: *mut NautilusInfoProvider,
        handle: *mut NautilusOperationHandle,
    );
    /// Invokes Nautilus' update-complete callback.
    fn nautilus_info_provider_update_complete_invoke(
        update_complete: *mut GClosure,
        provider: *mut NautilusInfoProvider,
        handle: *mut NautilusOperationHandle,
        result: NautilusOperationResult,
    );

    /// Returns the GType for `NautilusMenu`.
    fn nautilus_menu_get_type() -> GType;
    /// Appends a menu item to a menu.
    fn nautilus_menu_append_item(menu: *mut NautilusMenu, item: *mut NautilusMenuItem);
    /// Returns the items in a menu.
    fn nautilus_menu_get_items(menu: *mut NautilusMenu) -> *mut GList;
    /// Creates a new menu.
    fn nautilus_menu_new() -> *mut NautilusMenu;
    /// Returns the GType for `NautilusMenuItem`.
    fn nautilus_menu_item_get_type() -> GType;
    /// Frees a `GList` of `NautilusMenuItem` objects.
    fn nautilus_menu_item_list_free(item_list: *mut GList);
    /// Creates a new menu item.
    fn nautilus_menu_item_new(
        name: *const c_char,
        label: *const c_char,
        tip: *const c_char,
        icon: *const c_char,
    ) -> *mut NautilusMenuItem;
    /// Activates a menu item.
    fn nautilus_menu_item_activate(item: *mut NautilusMenuItem);
    /// Attaches a submenu to a menu item.
    fn nautilus_menu_item_set_submenu(item: *mut NautilusMenuItem, menu: *mut NautilusMenu);
    /// Emits the menu provider's `items-updated` signal.
    fn nautilus_menu_provider_emit_items_updated_signal(provider: *mut NautilusMenuProvider);
    /// Returns the GType for `NautilusMenuProvider`.
    fn nautilus_menu_provider_get_type() -> GType;
    /// Calls a menu provider's selected-file item method.
    fn nautilus_menu_provider_get_file_items(
        provider: *mut NautilusMenuProvider,
        files: *mut GList,
    ) -> *mut GList;
    /// Calls a menu provider's background item method.
    fn nautilus_menu_provider_get_background_items(
        provider: *mut NautilusMenuProvider,
        current_folder: *mut NautilusFileInfo,
    ) -> *mut GList;

    /// Returns the GType for `NautilusPropertiesItem`.
    fn nautilus_properties_item_get_type() -> GType;
    /// Creates a new properties item.
    fn nautilus_properties_item_new(
        name: *const c_char,
        value: *const c_char,
    ) -> *mut NautilusPropertiesItem;
    /// Returns a properties item's name.
    fn nautilus_properties_item_get_name(item: *mut NautilusPropertiesItem) -> *const c_char;
    /// Returns a properties item's value.
    fn nautilus_properties_item_get_value(item: *mut NautilusPropertiesItem) -> *const c_char;
    /// Returns the GType for `NautilusPropertiesModel`.
    fn nautilus_properties_model_get_type() -> GType;
    /// Creates a new properties model.
    fn nautilus_properties_model_new(
        title: *const c_char,
        model: *mut GListModel,
    ) -> *mut NautilusPropertiesModel;
    /// Returns the underlying list model.
    fn nautilus_properties_model_get_model(
        model: *mut NautilusPropertiesModel,
    ) -> *mut GListModel;
    /// Returns the properties model title.
    fn nautilus_properties_model_get_title(
        model: *mut NautilusPropertiesModel,
    ) -> *const c_char;
    /// Sets the properties model title.
    fn nautilus_properties_model_set_title(
        model: *mut NautilusPropertiesModel,
        title: *const c_char,
    );
    /// Returns the GType for `NautilusPropertiesModelProvider`.
    fn nautilus_properties_model_provider_get_type() -> GType;
    /// Calls a properties model provider's model method.
    fn nautilus_properties_model_provider_get_models(
        provider: *mut NautilusPropertiesModelProvider,
        files: *mut GList,
    ) -> *mut GList;
}

#[cfg(test)]
mod tests;
