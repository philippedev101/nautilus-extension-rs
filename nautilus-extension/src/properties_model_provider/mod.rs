use crate::gio_ffi::{
    g_list_model_get_item, g_list_model_get_n_items, g_list_store_append, g_list_store_new,
    GListModel,
};
use crate::glib_ffi::{g_list_append, g_list_free, gpointer, GList, GType};
use crate::gobject_ffi::{g_object_ref, g_object_unref, GObject, GObject as RawGObject};
use crate::gobject_utils::free_owned_g_object_list;
use crate::info_provider::{FileInfo, OwnedGObject};
use crate::nautilus_ffi::NATIVE_API_AVAILABLE;
use crate::nautilus_ffi::{
    nautilus_properties_item_get_name, nautilus_properties_item_get_type,
    nautilus_properties_item_get_value, nautilus_properties_item_new,
    nautilus_properties_model_get_model, nautilus_properties_model_get_title,
    nautilus_properties_model_get_type, nautilus_properties_model_new,
    nautilus_properties_model_provider_get_models, nautilus_properties_model_provider_get_type,
    nautilus_properties_model_set_title, NautilusPropertiesItem, NautilusPropertiesModel,
    NautilusPropertiesModelProvider, NautilusPropertiesModelProviderIface,
};
use crate::slot_allocator::{release_slot, reset_slots, take_next_slot};
use crate::translate::{borrowed_string, file_info_vec_from_g_list, vec_from_g_list};
use libc::c_void;
use std::borrow::Cow;
use std::ffi::CString;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

mod model;
pub use model::*;
mod provider;
pub use provider::*;
mod iface;
#[cfg(test)]
pub(crate) use iface::properties_model_provider_slot_is_set;
pub(crate) use iface::release_properties_model_provider_iface_index;
pub use iface::{
    properties_model_provider_iface_externs, reset_properties_model_provider_state,
    rust_properties_model_provider_setters, take_next_properties_model_provider_iface_index,
    MAX_PROPERTIES_MODEL_PROVIDERS,
};
#[cfg(test)]
mod tests;
