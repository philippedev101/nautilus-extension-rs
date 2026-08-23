use crate::glib_ffi::{g_list_append, g_list_free, gpointer, gulong, GList, GType};
use crate::gobject_ffi::{
    g_object_new, g_object_ref, g_object_unref, g_signal_connect_data, g_signal_handler_disconnect,
    GClosure, GObject,
};
use crate::gobject_utils::{
    get_bool_property, get_object_property, get_string_property, set_bool_property,
    set_optional_string_property, set_string_property,
};
use crate::info_provider::FileInfo;
use crate::nautilus_ffi::{
    nautilus_menu_append_item, nautilus_menu_get_items, nautilus_menu_get_type,
    nautilus_menu_item_activate, nautilus_menu_item_get_type, nautilus_menu_item_list_free,
    nautilus_menu_item_new, nautilus_menu_item_set_submenu, nautilus_menu_new,
    nautilus_menu_provider_emit_items_updated_signal, nautilus_menu_provider_get_background_items,
    nautilus_menu_provider_get_file_items, nautilus_menu_provider_get_type, NautilusFileInfo,
    NautilusMenu, NautilusMenuItem, NautilusMenuItemClass, NautilusMenuProvider,
    NautilusMenuProviderIface, NATIVE_API_AVAILABLE,
};
use crate::slot_allocator::{release_slot, reset_slots, take_next_slot};
use crate::translate::{file_info_vec_from_g_list, vec_from_g_list};
use libc::{c_char, c_void};
use std::borrow::Cow;
use std::ffi::CString;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

mod api;
pub use api::*;
mod menu_object;
pub use menu_object::*;
mod menu_item_object;
pub use menu_item_object::*;
mod activation;
#[allow(unused_imports)]
pub(crate) use activation::*;
pub use activation::{MenuActivation, MenuActivationTarget};
mod menu_item_class;
#[cfg(test)]
pub(crate) use menu_item_class::menu_item_activate_slot_is_set;
pub(crate) use menu_item_class::release_menu_item_class_index;
pub use menu_item_class::{
    menu_item_class_init_externs, reset_menu_item_activate_state, rust_menu_item_activate_setters,
    take_next_menu_item_class_index, MAX_MENU_ITEM_ACTIVATORS,
};
mod menu_provider_iface;
#[cfg(test)]
pub(crate) use menu_provider_iface::menu_provider_slot_is_set;
pub(crate) use menu_provider_iface::release_menu_provider_iface_index;
pub use menu_provider_iface::{
    menu_provider_iface_externs, reset_menu_provider_state, rust_menu_provider_setters,
    take_next_menu_provider_iface_index, MAX_MENU_PROVIDERS,
};
#[cfg(test)]
mod tests;
