use crate::gio_ffi::{
    g_file_get_path, g_file_get_uri, GFile, GFileType, GMount, G_FILE_TYPE_UNKNOWN,
};
use crate::glib_ffi::{
    g_idle_source_new, g_list_append, g_list_free, g_main_context_default, g_main_context_ref,
    g_main_context_ref_thread_default, g_main_context_unref, g_source_attach,
    g_source_set_callback, g_source_set_priority, g_source_unref, g_strdup, gboolean, gpointer,
    GList, GMainContext, GType, GFALSE, GTRUE, G_PRIORITY_DEFAULT,
};
use crate::gobject_ffi::{
    g_closure_ref, g_closure_unref, g_object_ref, g_object_unref, GClosure, GObject,
};
use crate::gobject_utils::NautilusObject;
use crate::nautilus_ffi::{
    nautilus_file_info_add_emblem, nautilus_file_info_add_string_attribute,
    nautilus_file_info_can_write, nautilus_file_info_create, nautilus_file_info_create_for_uri,
    nautilus_file_info_get_activation_uri, nautilus_file_info_get_file_type,
    nautilus_file_info_get_location, nautilus_file_info_get_mime_type,
    nautilus_file_info_get_mount, nautilus_file_info_get_name, nautilus_file_info_get_parent_info,
    nautilus_file_info_get_parent_location, nautilus_file_info_get_parent_uri,
    nautilus_file_info_get_string_attribute, nautilus_file_info_get_type,
    nautilus_file_info_get_uri, nautilus_file_info_get_uri_scheme,
    nautilus_file_info_invalidate_extension_info, nautilus_file_info_is_directory,
    nautilus_file_info_is_gone, nautilus_file_info_is_mime_type, nautilus_file_info_list_copy,
    nautilus_file_info_list_free, nautilus_file_info_lookup, nautilus_file_info_lookup_for_uri,
    nautilus_info_provider_cancel_update, nautilus_info_provider_get_type,
    nautilus_info_provider_update_complete_invoke, nautilus_info_provider_update_file_info,
    nautilus_operation_result_get_type, NautilusFileInfo, NautilusFileInfoInterface,
    NautilusInfoProvider, NautilusInfoProviderIface, NautilusOperationHandle,
    NautilusOperationResult, NATIVE_API_AVAILABLE,
};
use crate::slot_allocator::{release_slot, reset_slots, take_next_slot};
use crate::translate::{file_info_vec_from_g_list, take_glib_string};
use libc::c_char;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

mod api;
pub use api::*;
mod file_info;
pub use file_info::*;
mod async_update;
pub(crate) use async_update::*;
pub use async_update::{OperationHandle, PendingUpdate, UpdateCompletion, UpdateFileInfoOperation};
mod file_info_iface;
#[cfg(test)]
pub(crate) use file_info_iface::file_info_impl_slot_is_set;
pub(crate) use file_info_iface::release_file_info_iface_index;
pub use file_info_iface::{
    file_info_iface_externs, reset_file_info_impl_state, rust_file_info_impl_setters,
    take_next_file_info_iface_index, MAX_FILE_INFO_IMPLS,
};
mod info_provider_iface;
#[cfg(test)]
pub(crate) use info_provider_iface::info_provider_slot_is_set;
pub(crate) use info_provider_iface::release_info_provider_iface_index;
pub use info_provider_iface::{
    info_provider_iface_externs, reset_info_provider_state, rust_info_provider_setters,
    take_next_info_provider_iface_index, MAX_INFO_PROVIDERS,
};
#[cfg(test)]
mod tests;
