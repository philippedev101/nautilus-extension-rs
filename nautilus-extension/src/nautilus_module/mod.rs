use crate::column_provider::{
    column_provider_iface_externs, release_column_provider_iface_index,
    rust_column_provider_setters, take_next_column_provider_iface_index, ColumnProvider,
    MAX_COLUMN_PROVIDERS,
};
use crate::glib_ffi::GType;
use crate::gobject_ffi::{
    g_type_module_add_interface, g_type_module_register_type, g_type_query, GInterfaceInfo,
    GObjectClass, GTypeInfo, GTypeModule, GTypeQuery, GTypeValueTable, G_TYPE_OBJECT,
};
use crate::info_provider::{
    file_info_iface_externs, info_provider_iface_externs, release_file_info_iface_index,
    release_info_provider_iface_index, rust_file_info_impl_setters, rust_info_provider_setters,
    take_next_file_info_iface_index, take_next_info_provider_iface_index, FileInfoImpl,
    InfoProvider, MAX_FILE_INFO_IMPLS, MAX_INFO_PROVIDERS,
};
use crate::menu_provider::{
    menu_item_class_init_externs, menu_provider_iface_externs, release_menu_item_class_index,
    release_menu_provider_iface_index, rust_menu_item_activate_setters, rust_menu_provider_setters,
    take_next_menu_item_class_index, take_next_menu_provider_iface_index, MenuItemActivate,
    MenuItemType, MenuProvider, MAX_MENU_ITEM_ACTIVATORS, MAX_MENU_PROVIDERS,
};
use crate::nautilus_ffi::{
    nautilus_column_provider_get_type, nautilus_file_info_get_type,
    nautilus_info_provider_get_type, nautilus_menu_item_get_type, nautilus_menu_provider_get_type,
    nautilus_properties_model_provider_get_type, NATIVE_API_AVAILABLE,
};
use crate::properties_model_provider::{
    properties_model_provider_iface_externs, release_properties_model_provider_iface_index,
    rust_properties_model_provider_setters, take_next_properties_model_provider_iface_index,
    PropertiesModelProvider, MAX_PROPERTIES_MODEL_PROVIDERS,
};
use libc::c_char;
use std::borrow::Cow;
use std::cell::Cell;
use std::error::Error;
use std::ffi::CString;
use std::{fmt, mem, ptr};

#[repr(C)]
struct NautilusExtensionClass {
    _parent_slot: GObjectClass,
}

const EMPTY_VALUE_TABLE: GTypeValueTable = GTypeValueTable {
    value_init: None,
    value_free: None,
    value_copy: None,
    value_peek_pointer: None,
    collect_format: ptr::null::<c_char>(),
    collect_value: None,
    lcopy_format: ptr::null::<c_char>(),
    lcopy_value: None,
};

/// Builder used to register one Nautilus extension GType.
///
/// A module can implement any combination of the provider interfaces. Call
/// [`NautilusModule::register`] from the function passed to
/// [`crate::nautilus_module!`].
pub struct NautilusModule {
    module: *mut GTypeModule,
    name: Cow<'static, str>,
    column_provider_iface_infos: Vec<GInterfaceInfo>,
    file_info_iface_infos: Vec<GInterfaceInfo>,
    info_provider_iface_infos: Vec<GInterfaceInfo>,
    menu_provider_iface_infos: Vec<GInterfaceInfo>,
    properties_model_provider_iface_infos: Vec<GInterfaceInfo>,
    reservations: Vec<ProviderReservation>,
    registered: Cell<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProviderReservation {
    Column(usize),
    FileInfo(usize),
    Info(usize),
    MenuItemActivate(usize),
    Menu(usize),
    PropertiesModel(usize),
}

impl ProviderReservation {
    fn release(self) {
        match self {
            ProviderReservation::Column(index) => release_column_provider_iface_index(index),
            ProviderReservation::FileInfo(index) => release_file_info_iface_index(index),
            ProviderReservation::Info(index) => release_info_provider_iface_index(index),
            ProviderReservation::MenuItemActivate(index) => release_menu_item_class_index(index),
            ProviderReservation::Menu(index) => release_menu_provider_iface_index(index),
            ProviderReservation::PropertiesModel(index) => {
                release_properties_model_provider_iface_index(index)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Errors that can occur while preparing or registering a Nautilus module.
pub enum NautilusModuleError {
    /// No generated trampoline slot remains for this provider kind.
    TooManyProviders {
        /// Provider kind that exhausted its generated slots.
        provider_type: &'static str,
        /// Maximum number of supported providers for this kind.
        max: usize,
    },
    /// The same interface was added twice to one module type.
    DuplicateInterface {
        /// Interface that was already present.
        interface: &'static str,
    },
    /// The requested GType name contained an interior NUL byte.
    InvalidTypeName {
        /// Invalid type name.
        type_name: Cow<'static, str>,
    },
    /// The native Nautilus API is unavailable in this build.
    NativeApiUnavailable {
        /// API that could not be called.
        api: &'static str,
    },
    /// GObject rejected native type registration.
    TypeRegistrationFailed {
        /// Type name that failed to register.
        type_name: Cow<'static, str>,
    },
}

impl fmt::Display for NautilusModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NautilusModuleError::TooManyProviders { provider_type, max } => write!(
                f,
                "too many Nautilus {provider_type} providers registered; this build supports {max}"
            ),
            NautilusModuleError::DuplicateInterface { interface } => write!(
                f,
                "Nautilus {interface} interface is already registered on this module type"
            ),
            NautilusModuleError::InvalidTypeName { type_name } => {
                write!(
                    f,
                    "invalid GType name for Nautilus extension type: {type_name:?}"
                )
            }
            NautilusModuleError::NativeApiUnavailable { api } => {
                write!(f, "Nautilus native API is unavailable in this build: {api}")
            }
            NautilusModuleError::TypeRegistrationFailed { type_name } => write!(
                f,
                "failed to register Nautilus extension GType: {type_name}"
            ),
        }
    }
}

impl Error for NautilusModuleError {}

/// Values accepted from a `nautilus_module!` registration function.
///
/// Returning one `GType` keeps the original API working. Returning a `Vec<GType>`
/// or fixed-size array lets one extension shared object export multiple provider
/// classes through `nautilus_module_list_types`.
pub trait IntoModuleTypes {
    /// Converts the registration return value into the exported GType list.
    fn into_module_types(self) -> Vec<GType>;
}

impl IntoModuleTypes for GType {
    fn into_module_types(self) -> Vec<GType> {
        if self == 0 {
            Vec::new()
        } else {
            vec![self]
        }
    }
}

impl IntoModuleTypes for Option<GType> {
    fn into_module_types(self) -> Vec<GType> {
        self.unwrap_or(0).into_module_types()
    }
}

impl IntoModuleTypes for Vec<GType> {
    fn into_module_types(self) -> Vec<GType> {
        self.into_iter().filter(|type_| *type_ != 0).collect()
    }
}

impl<const N: usize> IntoModuleTypes for [GType; N] {
    fn into_module_types(self) -> Vec<GType> {
        self.into_iter().filter(|type_| *type_ != 0).collect()
    }
}

mod builder;
#[cfg(test)]
mod tests;
