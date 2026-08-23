use super::*;

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

        if !NATIVE_API_AVAILABLE {
            release_menu_item_class_index(index);
            return Err(NautilusModuleError::NativeApiUnavailable {
                api: "NautilusMenuItem subtype registration",
            });
        }

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
            g_type_module_register_type(self.module, parent_type, c_type_name.as_ptr(), &info, 0)
        };

        if item_type == 0 {
            release_menu_item_class_index(index);
            return Err(NautilusModuleError::TypeRegistrationFailed { type_name });
        }

        self.reservations
            .push(ProviderReservation::MenuItemActivate(index));

        Ok(unsafe { MenuItemType::from_raw(item_type) }.expect("registered GType is nonzero"))
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
        if !NATIVE_API_AVAILABLE {
            return 0;
        }

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
            let module_type =
                g_type_module_register_type(self.module, G_TYPE_OBJECT, name.as_ptr(), &info, 0);

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

fn g_object_instance_size() -> u16 {
    g_type_size(G_TYPE_OBJECT).instance_size
}

#[derive(Clone, Copy)]
struct GTypeSize {
    class_size: u16,
    instance_size: u16,
}

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
