use crate::column_provider::{
    column_provider_iface_externs, release_column_provider_iface_index,
    rust_column_provider_setters, take_next_column_provider_iface_index, ColumnProvider,
    MAX_COLUMN_PROVIDERS,
};
use crate::glib_ffi::GType;
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gobject_ffi::G_TYPE_OBJECT;
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gobject_ffi::{g_type_module_add_interface, g_type_module_register_type, g_type_query};
use crate::gobject_ffi::{GInterfaceInfo, GTypeModule};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gobject_ffi::{GObjectClass, GTypeInfo, GTypeQuery, GTypeValueTable};
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
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::nautilus_ffi::nautilus_menu_item_get_type;
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::nautilus_ffi::{
    nautilus_column_provider_get_type, nautilus_file_info_get_type,
    nautilus_info_provider_get_type, nautilus_menu_provider_get_type,
    nautilus_properties_model_provider_get_type,
};
use crate::properties_model_provider::{
    properties_model_provider_iface_externs, release_properties_model_provider_iface_index,
    rust_properties_model_provider_setters, take_next_properties_model_provider_iface_index,
    PropertiesModelProvider, MAX_PROPERTIES_MODEL_PROVIDERS,
};
#[cfg(not(nautilus_extension_rs_skip_link))]
use libc::c_char;
use std::borrow::Cow;
use std::cell::Cell;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
#[cfg(not(nautilus_extension_rs_skip_link))]
use std::mem;
use std::ptr;

#[cfg(not(nautilus_extension_rs_skip_link))]
#[repr(C)]
struct NautilusExtensionClass {
    _parent_slot: GObjectClass,
}

#[cfg(not(nautilus_extension_rs_skip_link))]
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
    #[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
    module: *mut GTypeModule,
    #[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
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
    #[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
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

impl NautilusModule {
    /// Creates a module registration builder.
    ///
    /// `name` must be a valid GType name when [`NautilusModule::register`] is
    /// called.
    pub fn new<S: Into<Cow<'static, str>>>(module: *mut GTypeModule, name: S) -> NautilusModule {
        NautilusModule {
            module,
            name: name.into(),
            column_provider_iface_infos: Vec::new(),
            file_info_iface_infos: Vec::new(),
            info_provider_iface_infos: Vec::new(),
            menu_provider_iface_infos: Vec::new(),
            properties_model_provider_iface_infos: Vec::new(),
            reservations: Vec::new(),
            registered: Cell::new(false),
        }
    }

    /// Adds a [`ColumnProvider`] to this module.
    ///
    /// # Panics
    ///
    /// Panics if the provider cannot be added. Use
    /// [`NautilusModule::try_add_column_provider`] to handle the error.
    pub fn add_column_provider<T: ColumnProvider + 'static>(
        &mut self,
        column_provider: T,
    ) -> &mut NautilusModule {
        self.try_add_column_provider(column_provider)
            .expect("failed to add Nautilus column provider")
    }

    /// Tries to add a [`ColumnProvider`] to this module.
    ///
    /// # Errors
    ///
    /// Returns an error if this module already has a column provider or the
    /// generated provider slots are exhausted.
    pub fn try_add_column_provider<T: ColumnProvider + 'static>(
        &mut self,
        column_provider: T,
    ) -> Result<&mut NautilusModule, NautilusModuleError> {
        if !self.column_provider_iface_infos.is_empty() {
            return Err(NautilusModuleError::DuplicateInterface {
                interface: "column provider",
            });
        }

        let index = take_next_column_provider_iface_index().ok_or(
            NautilusModuleError::TooManyProviders {
                provider_type: "column",
                max: MAX_COLUMN_PROVIDERS,
            },
        )?;
        let iface_init_fn = column_provider_iface_externs()[index];
        let rust_provider_setter = &rust_column_provider_setters()[index];

        let column_provider_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(column_provider));

        self.column_provider_iface_infos
            .push(column_provider_iface_info);
        self.reservations.push(ProviderReservation::Column(index));

        Ok(self)
    }

    /// Adds a Rust implementation of Nautilus' `FileInfo` interface.
    ///
    /// Most extensions do not need this; use [`InfoProvider`] when you only
    /// need to add attributes to Nautilus-owned files.
    ///
    /// # Panics
    ///
    /// Panics if the interface cannot be added. Use
    /// [`NautilusModule::try_add_file_info`] to handle the error.
    pub fn add_file_info<T: FileInfoImpl + 'static>(
        &mut self,
        file_info: T,
    ) -> &mut NautilusModule {
        self.try_add_file_info(file_info)
            .expect("failed to add Nautilus file info interface")
    }

    /// Tries to add a Rust implementation of Nautilus' `FileInfo` interface.
    ///
    /// # Errors
    ///
    /// Returns an error if this module already has a file-info implementation
    /// or the generated interface slots are exhausted.
    pub fn try_add_file_info<T: FileInfoImpl + 'static>(
        &mut self,
        file_info: T,
    ) -> Result<&mut NautilusModule, NautilusModuleError> {
        if !self.file_info_iface_infos.is_empty() {
            return Err(NautilusModuleError::DuplicateInterface {
                interface: "file info",
            });
        }

        let index =
            take_next_file_info_iface_index().ok_or(NautilusModuleError::TooManyProviders {
                provider_type: "file info",
                max: MAX_FILE_INFO_IMPLS,
            })?;
        let iface_init_fn = file_info_iface_externs()[index];
        let rust_file_info_setter = &rust_file_info_impl_setters()[index];

        let file_info_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_file_info_setter(Box::new(file_info));

        self.file_info_iface_infos.push(file_info_iface_info);
        self.reservations.push(ProviderReservation::FileInfo(index));

        Ok(self)
    }

    /// Adds an [`InfoProvider`] to this module.
    ///
    /// # Panics
    ///
    /// Panics if the provider cannot be added. Use
    /// [`NautilusModule::try_add_info_provider`] to handle the error.
    pub fn add_info_provider<T: InfoProvider + 'static>(
        &mut self,
        info_provider: T,
    ) -> &mut NautilusModule {
        self.try_add_info_provider(info_provider)
            .expect("failed to add Nautilus info provider")
    }

    /// Tries to add an [`InfoProvider`] to this module.
    ///
    /// # Errors
    ///
    /// Returns an error if this module already has an info provider or the
    /// generated provider slots are exhausted.
    pub fn try_add_info_provider<T: InfoProvider + 'static>(
        &mut self,
        info_provider: T,
    ) -> Result<&mut NautilusModule, NautilusModuleError> {
        if !self.info_provider_iface_infos.is_empty() {
            return Err(NautilusModuleError::DuplicateInterface {
                interface: "info provider",
            });
        }

        let index =
            take_next_info_provider_iface_index().ok_or(NautilusModuleError::TooManyProviders {
                provider_type: "info",
                max: MAX_INFO_PROVIDERS,
            })?;
        let iface_init_fn = info_provider_iface_externs()[index];
        let rust_provider_setter = &rust_info_provider_setters()[index];

        let info_provider_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(info_provider));

        self.info_provider_iface_infos
            .push(info_provider_iface_info);
        self.reservations.push(ProviderReservation::Info(index));

        Ok(self)
    }

    /// Registers a custom `NautilusMenuItem` subtype.
    ///
    /// # Panics
    ///
    /// Panics if the subtype cannot be registered. Use
    /// [`NautilusModule::try_register_menu_item_type`] to handle the error.
    pub fn register_menu_item_type<S, T>(&mut self, type_name: S, activate: T) -> MenuItemType
    where
        S: Into<Cow<'static, str>>,
        T: MenuItemActivate + 'static,
    {
        self.try_register_menu_item_type(type_name, activate)
            .expect("failed to register Nautilus menu item type")
    }

    /// Tries to register a custom `NautilusMenuItem` subtype.
    ///
    /// # Errors
    ///
    /// Returns an error if the type name is invalid, native registration is not
    /// available, native type registration fails, or the generated activator
    /// slots are exhausted.
    pub fn try_register_menu_item_type<S, T>(
        &mut self,
        type_name: S,
        activate: T,
    ) -> Result<MenuItemType, NautilusModuleError>
    where
        S: Into<Cow<'static, str>>,
        T: MenuItemActivate + 'static,
    {
        let type_name = type_name.into();
        let c_type_name =
            CString::new(type_name.as_ref()).map_err(|_| NautilusModuleError::InvalidTypeName {
                type_name: type_name.clone(),
            })?;

        let index =
            take_next_menu_item_class_index().ok_or(NautilusModuleError::TooManyProviders {
                provider_type: "menu item activator",
                max: MAX_MENU_ITEM_ACTIVATORS,
            })?;
        let class_init_fn = menu_item_class_init_externs()[index];
        let rust_activate_setter = &rust_menu_item_activate_setters()[index];

        rust_activate_setter(Box::new(activate));

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = c_type_name;
            let _ = class_init_fn;
            release_menu_item_class_index(index);
            Err(NautilusModuleError::NativeApiUnavailable {
                api: "NautilusMenuItem subtype registration",
            })
        }

        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            let parent_type = unsafe { nautilus_menu_item_get_type() };
            let parent_size = g_type_size(parent_type);
            let info = GTypeInfo {
                class_size: parent_size.class_size,
                base_init: None,
                base_finalize: None,
                class_init: Some(class_init_fn),
                class_finalize: None,
                class_data: ptr::null(),
                instance_size: parent_size.instance_size,
                n_preallocs: 0,
                instance_init: None,
                value_table: &EMPTY_VALUE_TABLE,
            };

            let item_type = unsafe {
                g_type_module_register_type(
                    self.module,
                    parent_type,
                    c_type_name.as_ptr(),
                    &info,
                    0,
                )
            };

            if item_type == 0 {
                release_menu_item_class_index(index);
                return Err(NautilusModuleError::TypeRegistrationFailed { type_name });
            }

            self.reservations
                .push(ProviderReservation::MenuItemActivate(index));

            Ok(unsafe { MenuItemType::from_raw(item_type) }.expect("registered GType is nonzero"))
        }
    }

    /// Adds a [`MenuProvider`] to this module.
    ///
    /// # Panics
    ///
    /// Panics if the provider cannot be added. Use
    /// [`NautilusModule::try_add_menu_provider`] to handle the error.
    pub fn add_menu_provider<T: MenuProvider + 'static>(
        &mut self,
        menu_provider: T,
    ) -> &mut NautilusModule {
        self.try_add_menu_provider(menu_provider)
            .expect("failed to add Nautilus menu provider")
    }

    /// Tries to add a [`MenuProvider`] to this module.
    ///
    /// # Errors
    ///
    /// Returns an error if this module already has a menu provider or the
    /// generated provider slots are exhausted.
    pub fn try_add_menu_provider<T: MenuProvider + 'static>(
        &mut self,
        menu_provider: T,
    ) -> Result<&mut NautilusModule, NautilusModuleError> {
        if !self.menu_provider_iface_infos.is_empty() {
            return Err(NautilusModuleError::DuplicateInterface {
                interface: "menu provider",
            });
        }

        let index =
            take_next_menu_provider_iface_index().ok_or(NautilusModuleError::TooManyProviders {
                provider_type: "menu",
                max: MAX_MENU_PROVIDERS,
            })?;
        let iface_init_fn = menu_provider_iface_externs()[index];
        let rust_provider_setter = &rust_menu_provider_setters()[index];

        let menu_provider_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(menu_provider));

        self.menu_provider_iface_infos
            .push(menu_provider_iface_info);
        self.reservations.push(ProviderReservation::Menu(index));

        Ok(self)
    }

    /// Adds a [`PropertiesModelProvider`] to this module.
    ///
    /// # Panics
    ///
    /// Panics if the provider cannot be added. Use
    /// [`NautilusModule::try_add_properties_model_provider`] to handle the
    /// error.
    pub fn add_properties_model_provider<T: PropertiesModelProvider + 'static>(
        &mut self,
        properties_model_provider: T,
    ) -> &mut NautilusModule {
        self.try_add_properties_model_provider(properties_model_provider)
            .expect("failed to add Nautilus properties model provider")
    }

    /// Tries to add a [`PropertiesModelProvider`] to this module.
    ///
    /// # Errors
    ///
    /// Returns an error if this module already has a properties-model provider
    /// or the generated provider slots are exhausted.
    pub fn try_add_properties_model_provider<T: PropertiesModelProvider + 'static>(
        &mut self,
        properties_model_provider: T,
    ) -> Result<&mut NautilusModule, NautilusModuleError> {
        if !self.properties_model_provider_iface_infos.is_empty() {
            return Err(NautilusModuleError::DuplicateInterface {
                interface: "properties model provider",
            });
        }

        let index = take_next_properties_model_provider_iface_index().ok_or(
            NautilusModuleError::TooManyProviders {
                provider_type: "properties model",
                max: MAX_PROPERTIES_MODEL_PROVIDERS,
            },
        )?;
        let iface_init_fn = properties_model_provider_iface_externs()[index];
        let rust_provider_setter = &rust_properties_model_provider_setters()[index];

        let iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(properties_model_provider));

        self.properties_model_provider_iface_infos.push(iface_info);
        self.reservations
            .push(ProviderReservation::PropertiesModel(index));

        Ok(self)
    }

    /// Registers the GObject type and attaches all configured interfaces.
    ///
    /// Returns `0` if native type registration is unavailable, the type name is
    /// invalid, or GObject rejects the registration.
    pub fn register(&self) -> GType {
        #[cfg(nautilus_extension_rs_skip_link)]
        {
            0
        }

        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            let name = match CString::new(&self.name as &str) {
                Ok(name) => name,
                Err(_) => return 0,
            };

            let info = GTypeInfo {
                class_size: mem::size_of::<NautilusExtensionClass>() as u16,
                base_init: None,
                base_finalize: None,
                class_init: None,
                class_finalize: None,
                class_data: ptr::null(),
                instance_size: g_object_instance_size(),
                n_preallocs: 0,
                instance_init: None,
                value_table: &EMPTY_VALUE_TABLE,
            };

            unsafe {
                let module_type = g_type_module_register_type(
                    self.module,
                    G_TYPE_OBJECT,
                    name.as_ptr(),
                    &info,
                    0,
                );

                if module_type == 0 {
                    return 0;
                }

                for column_provider_iface_info in &self.column_provider_iface_infos {
                    g_type_module_add_interface(
                        self.module,
                        module_type,
                        nautilus_column_provider_get_type(),
                        column_provider_iface_info,
                    );
                }

                for file_info_iface_info in &self.file_info_iface_infos {
                    g_type_module_add_interface(
                        self.module,
                        module_type,
                        nautilus_file_info_get_type(),
                        file_info_iface_info,
                    );
                }

                for info_provider_iface_info in &self.info_provider_iface_infos {
                    g_type_module_add_interface(
                        self.module,
                        module_type,
                        nautilus_info_provider_get_type(),
                        info_provider_iface_info,
                    );
                }

                for menu_provider_iface_info in &self.menu_provider_iface_infos {
                    g_type_module_add_interface(
                        self.module,
                        module_type,
                        nautilus_menu_provider_get_type(),
                        menu_provider_iface_info,
                    );
                }

                for iface_info in &self.properties_model_provider_iface_infos {
                    g_type_module_add_interface(
                        self.module,
                        module_type,
                        nautilus_properties_model_provider_get_type(),
                        iface_info,
                    );
                }

                self.registered.set(true);
                module_type
            }
        }
    }
}

impl Drop for NautilusModule {
    fn drop(&mut self) {
        if self.registered.get() {
            return;
        }

        while let Some(reservation) = self.reservations.pop() {
            reservation.release();
        }
    }
}

#[cfg(not(nautilus_extension_rs_skip_link))]
fn g_object_instance_size() -> u16 {
    g_type_size(G_TYPE_OBJECT).instance_size
}

#[derive(Clone, Copy)]
#[cfg(not(nautilus_extension_rs_skip_link))]
struct GTypeSize {
    class_size: u16,
    instance_size: u16,
}

#[cfg(not(nautilus_extension_rs_skip_link))]
fn g_type_size(type_: GType) -> GTypeSize {
    let mut query: GTypeQuery = GTypeQuery {
        type_: 0,
        type_name: ptr::null::<c_char>(),
        class_size: 0,
        instance_size: 0,
    };
    unsafe {
        g_type_query(type_, &mut query);
    }

    GTypeSize {
        class_size: query.class_size as u16,
        instance_size: query.instance_size as u16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Column, ColumnProvider, FileInfo, FileInfoImpl, InfoProvider, MenuItem, MenuItemObject,
        MenuProvider, PropertiesModel, PropertiesModelProvider,
    };

    struct TestColumnProvider;

    impl ColumnProvider for TestColumnProvider {
        fn get_columns(&self) -> Vec<Column> {
            Vec::new()
        }
    }

    struct TestInfoProvider;

    impl InfoProvider for TestInfoProvider {}

    struct TestFileInfoImpl;

    impl FileInfoImpl for TestFileInfoImpl {}

    struct TestMenuProvider;

    impl MenuProvider for TestMenuProvider {
        fn get_file_items(&self, _files: &[FileInfo]) -> Vec<MenuItem> {
            Vec::new()
        }
    }

    struct TestPropertiesModelProvider;

    impl PropertiesModelProvider for TestPropertiesModelProvider {
        fn get_models(&self, _files: &[FileInfo]) -> Vec<PropertiesModel> {
            Vec::new()
        }
    }

    #[test]
    fn module_starts_without_registered_interfaces() {
        let module = NautilusModule::new(ptr::null_mut(), "RustTestModule");

        assert_eq!(module.name.as_ref(), "RustTestModule");
        assert!(module.column_provider_iface_infos.is_empty());
        assert!(module.file_info_iface_infos.is_empty());
        assert!(module.info_provider_iface_infos.is_empty());
        assert!(module.menu_provider_iface_infos.is_empty());
        assert!(module.properties_model_provider_iface_infos.is_empty());
    }

    #[test]
    fn try_add_provider_methods_record_interface_info() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_item_activate_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();

        let mut module = NautilusModule::new(ptr::null_mut(), "RustTestProviders");

        module
            .try_add_column_provider(TestColumnProvider)
            .unwrap()
            .try_add_file_info(TestFileInfoImpl)
            .unwrap()
            .try_add_info_provider(TestInfoProvider)
            .unwrap()
            .try_add_menu_provider(TestMenuProvider)
            .unwrap()
            .try_add_properties_model_provider(TestPropertiesModelProvider)
            .unwrap();

        assert_eq!(module.column_provider_iface_infos.len(), 1);
        assert_eq!(module.file_info_iface_infos.len(), 1);
        assert_eq!(module.info_provider_iface_infos.len(), 1);
        assert_eq!(module.menu_provider_iface_infos.len(), 1);
        assert_eq!(module.properties_model_provider_iface_infos.len(), 1);

        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_item_activate_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();
    }

    #[test]
    fn module_error_describes_provider_limit() {
        let error = NautilusModuleError::TooManyProviders {
            provider_type: "menu",
            max: 10,
        };

        assert_eq!(
            error.to_string(),
            "too many Nautilus menu providers registered; this build supports 10"
        );

        let error = NautilusModuleError::DuplicateInterface {
            interface: "menu provider",
        };

        assert_eq!(
            error.to_string(),
            "Nautilus menu provider interface is already registered on this module type"
        );
    }

    #[test]
    fn module_error_describes_new_registration_failures() {
        let invalid = NautilusModuleError::InvalidTypeName {
            type_name: Cow::Borrowed("Bad\0Type"),
        };
        let unavailable = NautilusModuleError::NativeApiUnavailable {
            api: "NautilusMenuItem subtype registration",
        };
        let failed = NautilusModuleError::TypeRegistrationFailed {
            type_name: Cow::Borrowed("ExampleMenuItem"),
        };

        assert_eq!(
            invalid.to_string(),
            "invalid GType name for Nautilus extension type: \"Bad\\0Type\""
        );
        assert_eq!(
            unavailable.to_string(),
            "Nautilus native API is unavailable in this build: NautilusMenuItem subtype registration"
        );
        assert_eq!(
            failed.to_string(),
            "failed to register Nautilus extension GType: ExampleMenuItem"
        );
    }

    #[test]
    fn menu_item_type_registration_rejects_invalid_type_names_before_reserving_slot() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::menu_provider::reset_menu_item_activate_state();

        let mut module = NautilusModule::new(ptr::null_mut(), "RustInvalidMenuItemType");
        let error = module
            .try_register_menu_item_type("Bad\0Type", |_: &MenuItemObject| {})
            .unwrap_err();

        assert_eq!(
            error,
            NautilusModuleError::InvalidTypeName {
                type_name: Cow::Borrowed("Bad\0Type"),
            }
        );
        assert_eq!(take_next_menu_item_class_index(), Some(0));

        crate::menu_provider::reset_menu_item_activate_state();
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn menu_item_type_registration_reports_unavailable_native_api_in_no_link_mode() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::menu_provider::reset_menu_item_activate_state();

        let mut module = NautilusModule::new(ptr::null_mut(), "RustUnavailableMenuItemType");
        let error = module
            .try_register_menu_item_type("RustUnavailableMenuItem", |_: &MenuItemObject| {})
            .unwrap_err();

        assert_eq!(
            error,
            NautilusModuleError::NativeApiUnavailable {
                api: "NautilusMenuItem subtype registration",
            }
        );
        assert!(!crate::menu_provider::menu_item_activate_slot_is_set(0));
        assert_eq!(take_next_menu_item_class_index(), Some(0));

        crate::menu_provider::reset_menu_item_activate_state();
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn register_is_inert_and_releases_slots_in_no_link_mode() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();

        {
            let mut module = NautilusModule::new(ptr::null_mut(), "RustNoLinkRegister");
            module
                .try_add_column_provider(TestColumnProvider)
                .unwrap()
                .try_add_file_info(TestFileInfoImpl)
                .unwrap()
                .try_add_info_provider(TestInfoProvider)
                .unwrap()
                .try_add_menu_provider(TestMenuProvider)
                .unwrap()
                .try_add_properties_model_provider(TestPropertiesModelProvider)
                .unwrap();

            assert_eq!(module.register(), 0);
            assert!(!module.registered.get());
        }

        assert!(!crate::column_provider::column_provider_slot_is_set(0));
        assert!(!crate::info_provider::file_info_impl_slot_is_set(0));
        assert!(!crate::info_provider::info_provider_slot_is_set(0));
        assert!(!crate::menu_provider::menu_provider_slot_is_set(0));
        assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(0));

        assert_eq!(take_next_column_provider_iface_index(), Some(0));
        assert_eq!(take_next_file_info_iface_index(), Some(0));
        assert_eq!(take_next_info_provider_iface_index(), Some(0));
        assert_eq!(take_next_menu_provider_iface_index(), Some(0));
        assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();
    }

    #[test]
    fn unregistered_module_drop_releases_reserved_provider_slots() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_item_activate_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();

        {
            let mut module = NautilusModule::new(ptr::null_mut(), "RustRollbackProviders");
            module
                .try_add_column_provider(TestColumnProvider)
                .unwrap()
                .try_add_file_info(TestFileInfoImpl)
                .unwrap()
                .try_add_info_provider(TestInfoProvider)
                .unwrap()
                .try_add_menu_provider(TestMenuProvider)
                .unwrap()
                .try_add_properties_model_provider(TestPropertiesModelProvider)
                .unwrap();

            assert!(crate::column_provider::column_provider_slot_is_set(0));
            assert!(crate::info_provider::file_info_impl_slot_is_set(0));
            assert!(crate::info_provider::info_provider_slot_is_set(0));
            assert!(crate::menu_provider::menu_provider_slot_is_set(0));
            assert!(crate::properties_model_provider::properties_model_provider_slot_is_set(0));
        }

        assert!(!crate::column_provider::column_provider_slot_is_set(0));
        assert!(!crate::info_provider::file_info_impl_slot_is_set(0));
        assert!(!crate::info_provider::info_provider_slot_is_set(0));
        assert!(!crate::menu_provider::menu_item_activate_slot_is_set(0));
        assert!(!crate::menu_provider::menu_provider_slot_is_set(0));
        assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(0));

        assert_eq!(take_next_column_provider_iface_index(), Some(0));
        assert_eq!(take_next_file_info_iface_index(), Some(0));
        assert_eq!(take_next_info_provider_iface_index(), Some(0));
        assert_eq!(take_next_menu_item_class_index(), Some(0));
        assert_eq!(take_next_menu_provider_iface_index(), Some(0));
        assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_item_activate_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();
    }

    #[test]
    fn unregistered_module_drop_reuses_out_of_order_provider_slots() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();

        let mut first = NautilusModule::new(ptr::null_mut(), "RustOutOfOrderProvidersFirst");
        first
            .try_add_column_provider(TestColumnProvider)
            .unwrap()
            .try_add_file_info(TestFileInfoImpl)
            .unwrap()
            .try_add_info_provider(TestInfoProvider)
            .unwrap()
            .try_add_menu_provider(TestMenuProvider)
            .unwrap()
            .try_add_properties_model_provider(TestPropertiesModelProvider)
            .unwrap();

        let mut second = NautilusModule::new(ptr::null_mut(), "RustOutOfOrderProvidersSecond");
        second
            .try_add_column_provider(TestColumnProvider)
            .unwrap()
            .try_add_file_info(TestFileInfoImpl)
            .unwrap()
            .try_add_info_provider(TestInfoProvider)
            .unwrap()
            .try_add_menu_provider(TestMenuProvider)
            .unwrap()
            .try_add_properties_model_provider(TestPropertiesModelProvider)
            .unwrap();

        drop(first);

        assert!(!crate::column_provider::column_provider_slot_is_set(0));
        assert!(!crate::info_provider::file_info_impl_slot_is_set(0));
        assert!(!crate::info_provider::info_provider_slot_is_set(0));
        assert!(!crate::menu_provider::menu_provider_slot_is_set(0));
        assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(0));

        let mut third = NautilusModule::new(ptr::null_mut(), "RustOutOfOrderProvidersThird");
        third
            .try_add_column_provider(TestColumnProvider)
            .unwrap()
            .try_add_file_info(TestFileInfoImpl)
            .unwrap()
            .try_add_info_provider(TestInfoProvider)
            .unwrap()
            .try_add_menu_provider(TestMenuProvider)
            .unwrap()
            .try_add_properties_model_provider(TestPropertiesModelProvider)
            .unwrap();

        assert!(crate::column_provider::column_provider_slot_is_set(0));
        assert!(crate::info_provider::file_info_impl_slot_is_set(0));
        assert!(crate::info_provider::info_provider_slot_is_set(0));
        assert!(crate::menu_provider::menu_provider_slot_is_set(0));
        assert!(crate::properties_model_provider::properties_model_provider_slot_is_set(0));
        assert!(crate::column_provider::column_provider_slot_is_set(1));
        assert!(crate::info_provider::file_info_impl_slot_is_set(1));
        assert!(crate::info_provider::info_provider_slot_is_set(1));
        assert!(crate::menu_provider::menu_provider_slot_is_set(1));
        assert!(crate::properties_model_provider::properties_model_provider_slot_is_set(1));

        drop(second);
        drop(third);

        assert_eq!(take_next_column_provider_iface_index(), Some(0));
        assert_eq!(take_next_file_info_iface_index(), Some(0));
        assert_eq!(take_next_info_provider_iface_index(), Some(0));
        assert_eq!(take_next_menu_provider_iface_index(), Some(0));
        assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();
    }

    #[test]
    fn duplicate_provider_interfaces_are_rejected_without_reserving_slots() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();

        {
            let mut module = NautilusModule::new(ptr::null_mut(), "RustDuplicateProviders");

            module.try_add_column_provider(TestColumnProvider).unwrap();
            let error = match module.try_add_column_provider(TestColumnProvider) {
                Ok(_) => panic!("duplicate column provider unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::DuplicateInterface {
                    interface: "column provider",
                }
            );
            assert!(!crate::column_provider::column_provider_slot_is_set(1));

            module.try_add_file_info(TestFileInfoImpl).unwrap();
            let error = match module.try_add_file_info(TestFileInfoImpl) {
                Ok(_) => panic!("duplicate file info interface unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::DuplicateInterface {
                    interface: "file info",
                }
            );
            assert!(!crate::info_provider::file_info_impl_slot_is_set(1));

            module.try_add_info_provider(TestInfoProvider).unwrap();
            let error = match module.try_add_info_provider(TestInfoProvider) {
                Ok(_) => panic!("duplicate info provider unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::DuplicateInterface {
                    interface: "info provider",
                }
            );
            assert!(!crate::info_provider::info_provider_slot_is_set(1));

            module.try_add_menu_provider(TestMenuProvider).unwrap();
            let error = match module.try_add_menu_provider(TestMenuProvider) {
                Ok(_) => panic!("duplicate menu provider unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::DuplicateInterface {
                    interface: "menu provider",
                }
            );
            assert!(!crate::menu_provider::menu_provider_slot_is_set(1));

            module
                .try_add_properties_model_provider(TestPropertiesModelProvider)
                .unwrap();
            let error = match module.try_add_properties_model_provider(TestPropertiesModelProvider)
            {
                Ok(_) => panic!("duplicate properties model provider unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::DuplicateInterface {
                    interface: "properties model provider",
                }
            );
            assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(1));
        }

        assert_eq!(take_next_column_provider_iface_index(), Some(0));
        assert_eq!(take_next_file_info_iface_index(), Some(0));
        assert_eq!(take_next_info_provider_iface_index(), Some(0));
        assert_eq!(take_next_menu_provider_iface_index(), Some(0));
        assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

        crate::column_provider::reset_column_provider_state();
        crate::info_provider::reset_file_info_impl_state();
        crate::info_provider::reset_info_provider_state();
        crate::menu_provider::reset_menu_provider_state();
        crate::properties_model_provider::reset_properties_model_provider_state();
    }

    #[test]
    fn failed_try_add_does_not_advance_column_provider_slot_counter() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::column_provider::reset_column_provider_state();

        {
            let mut modules = Vec::new();
            for index in 0..MAX_COLUMN_PROVIDERS {
                let mut module =
                    NautilusModule::new(ptr::null_mut(), format!("RustColumnOverflow{index}"));
                module.try_add_column_provider(TestColumnProvider).unwrap();
                modules.push(module);
            }

            let mut module = NautilusModule::new(ptr::null_mut(), "RustColumnOverflowExtra");
            let error = match module.try_add_column_provider(TestColumnProvider) {
                Ok(_) => panic!("column provider overflow unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::TooManyProviders {
                    provider_type: "column",
                    max: MAX_COLUMN_PROVIDERS,
                }
            );

            while let Some(module) = modules.pop() {
                drop(module);
            }
        }

        assert_eq!(take_next_column_provider_iface_index(), Some(0));
        crate::column_provider::reset_column_provider_state();
    }

    #[test]
    fn failed_try_add_does_not_advance_file_info_slot_counter() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::info_provider::reset_file_info_impl_state();

        {
            let mut modules = Vec::new();
            for index in 0..MAX_FILE_INFO_IMPLS {
                let mut module =
                    NautilusModule::new(ptr::null_mut(), format!("RustFileInfoOverflow{index}"));
                module.try_add_file_info(TestFileInfoImpl).unwrap();
                modules.push(module);
            }

            let mut module = NautilusModule::new(ptr::null_mut(), "RustFileInfoOverflowExtra");
            let error = match module.try_add_file_info(TestFileInfoImpl) {
                Ok(_) => panic!("file info overflow unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::TooManyProviders {
                    provider_type: "file info",
                    max: MAX_FILE_INFO_IMPLS,
                }
            );

            while let Some(module) = modules.pop() {
                drop(module);
            }
        }

        assert_eq!(take_next_file_info_iface_index(), Some(0));
        crate::info_provider::reset_file_info_impl_state();
    }

    #[test]
    fn failed_try_add_does_not_advance_info_provider_slot_counter() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::info_provider::reset_info_provider_state();

        {
            let mut modules = Vec::new();
            for index in 0..MAX_INFO_PROVIDERS {
                let mut module =
                    NautilusModule::new(ptr::null_mut(), format!("RustInfoOverflow{index}"));
                module.try_add_info_provider(TestInfoProvider).unwrap();
                modules.push(module);
            }

            let mut module = NautilusModule::new(ptr::null_mut(), "RustInfoOverflowExtra");
            let error = match module.try_add_info_provider(TestInfoProvider) {
                Ok(_) => panic!("info provider overflow unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::TooManyProviders {
                    provider_type: "info",
                    max: MAX_INFO_PROVIDERS,
                }
            );

            while let Some(module) = modules.pop() {
                drop(module);
            }
        }

        assert_eq!(take_next_info_provider_iface_index(), Some(0));
        crate::info_provider::reset_info_provider_state();
    }

    #[test]
    fn failed_try_add_does_not_advance_menu_provider_slot_counter() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::menu_provider::reset_menu_provider_state();

        {
            let mut modules = Vec::new();
            for index in 0..MAX_MENU_PROVIDERS {
                let mut module =
                    NautilusModule::new(ptr::null_mut(), format!("RustMenuOverflow{index}"));
                module.try_add_menu_provider(TestMenuProvider).unwrap();
                modules.push(module);
            }

            let mut module = NautilusModule::new(ptr::null_mut(), "RustMenuOverflowExtra");
            let error = match module.try_add_menu_provider(TestMenuProvider) {
                Ok(_) => panic!("menu provider overflow unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::TooManyProviders {
                    provider_type: "menu",
                    max: MAX_MENU_PROVIDERS,
                }
            );

            while let Some(module) = modules.pop() {
                drop(module);
            }
        }

        assert_eq!(take_next_menu_provider_iface_index(), Some(0));
        crate::menu_provider::reset_menu_provider_state();
    }

    #[test]
    fn failed_try_add_does_not_advance_properties_model_provider_slot_counter() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        crate::properties_model_provider::reset_properties_model_provider_state();

        {
            let mut modules = Vec::new();
            for index in 0..MAX_PROPERTIES_MODEL_PROVIDERS {
                let mut module =
                    NautilusModule::new(ptr::null_mut(), format!("RustPropertiesOverflow{index}"));
                module
                    .try_add_properties_model_provider(TestPropertiesModelProvider)
                    .unwrap();
                modules.push(module);
            }

            let mut module = NautilusModule::new(ptr::null_mut(), "RustPropertiesOverflowExtra");
            let error = match module.try_add_properties_model_provider(TestPropertiesModelProvider)
            {
                Ok(_) => panic!("properties model provider overflow unexpectedly succeeded"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                NautilusModuleError::TooManyProviders {
                    provider_type: "properties model",
                    max: MAX_PROPERTIES_MODEL_PROVIDERS,
                }
            );

            while let Some(module) = modules.pop() {
                drop(module);
            }
        }

        assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));
        crate::properties_model_provider::reset_properties_model_provider_state();
    }

    #[test]
    fn module_type_conversion_accepts_single_optional_vec_and_array() {
        assert_eq!(IntoModuleTypes::into_module_types(7 as GType), vec![7]);
        assert!(IntoModuleTypes::into_module_types(0 as GType).is_empty());
        assert_eq!(
            IntoModuleTypes::into_module_types(Some(9 as GType)),
            vec![9]
        );
        assert!(IntoModuleTypes::into_module_types(None::<GType>).is_empty());
        assert_eq!(
            IntoModuleTypes::into_module_types(vec![1 as GType, 0 as GType, 2 as GType]),
            vec![1, 2]
        );
        assert_eq!(
            IntoModuleTypes::into_module_types([3 as GType, 0 as GType, 4 as GType]),
            vec![3, 4]
        );
    }
}
