use super::*;

#[derive(Debug)]
/// Owned reference to a `NautilusMenu` object.
pub struct MenuObject {
    pub(crate) raw: *mut NautilusMenu,
}

impl MenuObject {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusMenu` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_menu_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusMenu` GType.
    pub fn type_() -> GType {
        0
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Creates an empty native `NautilusMenu` object.
    pub fn new() -> Option<MenuObject> {
        unsafe { MenuObject::from_raw_full(nautilus_menu_new()) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Creates an empty native `NautilusMenu` object.
    pub fn new() -> Option<MenuObject> {
        None
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusMenu` GObject pointer. This
    /// function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(raw: *mut NautilusMenu) -> Option<MenuObject> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut GObject);
        }
        Some(MenuObject { raw })
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer `NautilusMenu`
    /// GObject pointer. On success, the returned wrapper owns that reference.
    pub unsafe fn from_raw_full(raw: *mut NautilusMenu) -> Option<MenuObject> {
        if raw.is_null() {
            return None;
        }

        Some(MenuObject { raw })
    }

    /// Returns the wrapped raw `NautilusMenu` pointer.
    pub fn raw(&self) -> *mut NautilusMenu {
        self.raw
    }

    /// Returns the wrapped raw `NautilusMenu` pointer.
    pub fn as_ptr(&self) -> *mut NautilusMenu {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusMenu {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Appends an item to this native menu.
    pub fn append_item(&self, item: &MenuItemObject) {
        unsafe {
            nautilus_menu_append_item(self.raw, item.raw());
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Appends an item to this native menu.
    pub fn append_item(&self, _item: &MenuItemObject) {}

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the native menu items currently in this menu.
    pub fn get_items(&self) -> Vec<MenuItemObject> {
        let items = unsafe { nautilus_menu_get_items(self.raw) };

        unsafe { MenuItemList::from_raw_full(items) }
            .map(|items| items.items())
            .unwrap_or_default()
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the native menu items currently in this menu.
    pub fn get_items(&self) -> Vec<MenuItemObject> {
        Vec::new()
    }

    /// Returns the native menu items currently in this menu.
    ///
    /// This is the Rust-style alias for [`MenuObject::get_items`].
    pub fn items(&self) -> Vec<MenuItemObject> {
        self.get_items()
    }
}

impl Clone for MenuObject {
    fn clone(&self) -> MenuObject {
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        MenuObject { raw: self.raw }
    }
}

impl Drop for MenuObject {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut GObject);
            }
        }
    }
}

#[derive(Debug)]
/// Owned full-transfer list of `NautilusMenuItem` objects.
pub struct MenuItemList {
    raw: *mut GList,
}

impl MenuItemList {
    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer `GList` containing
    /// `NautilusMenuItem` pointers. On success, the returned wrapper owns the
    /// list and will release it with `nautilus_menu_item_list_free`.
    pub unsafe fn from_raw_full(raw: *mut GList) -> Option<MenuItemList> {
        if raw.is_null() {
            None
        } else {
            Some(MenuItemList { raw })
        }
    }

    /// Returns the wrapped raw `GList` pointer.
    pub fn raw(&self) -> *mut GList {
        self.raw
    }

    /// Returns the wrapped raw `GList` pointer.
    pub fn as_ptr(&self) -> *mut GList {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw list pointer.
    pub fn into_raw(mut self) -> *mut GList {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    /// Returns the menu item objects contained in the list.
    pub fn items(&self) -> Vec<MenuItemObject> {
        unsafe {
            vec_from_g_list(self.raw, |data| {
                MenuItemObject::from_raw_borrowed(data as *mut NautilusMenuItem)
            })
        }
    }
}

impl Drop for MenuItemList {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            #[cfg(not(nautilus_extension_rs_skip_link))]
            unsafe {
                nautilus_menu_item_list_free(self.raw);
            }
        }
    }
}

/// A context-menu item.
pub struct MenuItem {
    pub(crate) item_type: Option<MenuItemType>,
    pub(crate) name: Cow<'static, str>,
    pub(crate) label: Cow<'static, str>,
    pub(crate) tip: Option<Cow<'static, str>>,
    pub(crate) icon: Option<Cow<'static, str>>,
    pub(crate) sensitive: bool,
    pub(crate) priority: Option<bool>,
    pub(crate) submenu: Option<Menu>,
    pub(crate) activate_fn: Option<Arc<dyn Fn(MenuActivation)>>,
}

impl Clone for MenuItem {
    fn clone(&self) -> MenuItem {
        MenuItem {
            item_type: self.item_type,
            name: self.name.clone(),
            label: self.label.clone(),
            tip: self.tip.clone(),
            icon: self.icon.clone(),
            sensitive: self.sensitive,
            priority: self.priority,
            submenu: self.submenu.clone(),
            activate_fn: self.activate_fn.clone(),
        }
    }
}

impl MenuItem {
    /// Creates a menu item with a unique name and user-visible label.
    pub fn new<N, L>(name: N, label: L) -> MenuItem
    where
        N: Into<Cow<'static, str>>,
        L: Into<Cow<'static, str>>,
    {
        MenuItem {
            item_type: None,
            name: name.into(),
            label: label.into(),
            tip: None,
            icon: None,
            sensitive: true,
            priority: None,
            submenu: None,
            activate_fn: None,
        }
    }

    /// Uses a custom `NautilusMenuItem` subtype for this item.
    pub fn with_menu_item_type(mut self, item_type: MenuItemType) -> MenuItem {
        self.item_type = Some(item_type);
        self
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item tips; this is kept for compatibility"
    )]
    /// Sets the deprecated menu item tip.
    pub fn with_tip<T: Into<Cow<'static, str>>>(mut self, tip: T) -> MenuItem {
        self.tip = Some(tip.into());
        self
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item icons; this is kept for compatibility"
    )]
    /// Sets the deprecated menu item icon name.
    pub fn with_icon<I: Into<Cow<'static, str>>>(mut self, icon: I) -> MenuItem {
        self.icon = Some(icon.into());
        self
    }

    /// Sets whether the menu item should be sensitive.
    pub fn sensitive(mut self, sensitive: bool) -> MenuItem {
        self.sensitive = sensitive;
        self
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates the menu item priority property; this is kept for compatibility"
    )]
    /// Sets the deprecated priority flag.
    pub fn priority(mut self, priority: bool) -> MenuItem {
        self.priority = Some(priority);
        self
    }

    /// Attaches a submenu to this item.
    pub fn with_submenu(mut self, submenu: Menu) -> MenuItem {
        self.submenu = Some(submenu);
        self
    }

    /// Sets a Rust callback to run when this item is activated.
    pub fn on_activate<F>(mut self, activate_fn: F) -> MenuItem
    where
        F: Fn(MenuActivation) + 'static,
    {
        self.activate_fn = Some(Arc::new(activate_fn));
        self
    }

    /// Returns the unique menu item name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the custom item subtype, if one was configured.
    pub fn item_type(&self) -> Option<MenuItemType> {
        self.item_type
    }

    /// Returns the user-visible menu item label.
    pub fn label(&self) -> &str {
        &self.label
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item tips; this is kept for compatibility"
    )]
    /// Returns the deprecated menu item tip.
    pub fn tip(&self) -> Option<&str> {
        self.tip.as_deref()
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item icons; this is kept for compatibility"
    )]
    /// Returns the deprecated menu item icon name.
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Returns whether the menu item should be sensitive.
    pub fn is_sensitive(&self) -> bool {
        self.sensitive
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates the menu item priority property; this is kept for compatibility"
    )]
    /// Returns the deprecated priority flag, if one was configured.
    pub fn priority_value(&self) -> Option<bool> {
        self.priority
    }

    /// Returns the attached submenu, if any.
    pub fn submenu(&self) -> Option<&Menu> {
        self.submenu.as_ref()
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Builds the corresponding native `NautilusMenuItem` object.
    pub fn to_object(&self, target: &MenuActivationTarget) -> Option<MenuItemObject> {
        unsafe { MenuItemObject::from_raw_full(self.to_raw(target)?) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Builds the corresponding native `NautilusMenuItem` object.
    pub fn to_object(&self, _target: &MenuActivationTarget) -> Option<MenuItemObject> {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    pub(crate) fn to_raw(&self, target: &MenuActivationTarget) -> Option<*mut NautilusMenuItem> {
        let name = CString::new(&self.name as &str).ok()?;
        let label = CString::new(&self.label as &str).ok()?;
        let tip = self
            .tip
            .as_ref()
            .and_then(|tip| CString::new(tip.as_ref()).ok());
        let icon = self
            .icon
            .as_ref()
            .and_then(|icon| CString::new(icon.as_ref()).ok());

        let raw_menuitem =
            new_menu_item_raw(self.item_type, &name, &label, tip.as_ref(), icon.as_ref())?;

        if raw_menuitem.is_null() {
            return None;
        }

        unsafe {
            set_bool_property(raw_menuitem as *mut GObject, "sensitive", self.sensitive);

            if let Some(priority) = self.priority {
                set_bool_property(raw_menuitem as *mut GObject, "priority", priority);
            }
        }

        if let Some(submenu) = &self.submenu {
            let raw_submenu = submenu.to_raw(target);
            if !raw_submenu.is_null() {
                unsafe {
                    nautilus_menu_item_set_submenu(raw_menuitem, raw_submenu);
                    g_object_unref(raw_submenu as *mut GObject);
                }
            }
        }

        if let Some(activate_fn) = &self.activate_fn {
            connect_activate_signal(raw_menuitem, activate_fn.clone(), target.clone());
        }

        Some(raw_menuitem)
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    pub(crate) fn to_raw(&self, _target: &MenuActivationTarget) -> Option<*mut NautilusMenuItem> {
        None
    }
}

#[cfg(not(nautilus_extension_rs_skip_link))]
pub(crate) fn new_menu_item_raw(
    item_type: Option<MenuItemType>,
    name: &CString,
    label: &CString,
    tip: Option<&CString>,
    icon: Option<&CString>,
) -> Option<*mut NautilusMenuItem> {
    let raw_menuitem = match item_type {
        Some(item_type) => {
            let name_property = CString::new("name").ok()?;
            let label_property = CString::new("label").ok()?;
            let tip_property = CString::new("tip").ok()?;
            let icon_property = CString::new("icon").ok()?;

            unsafe {
                g_object_new(
                    item_type.raw(),
                    name_property.as_ptr(),
                    name.as_ptr(),
                    label_property.as_ptr(),
                    label.as_ptr(),
                    tip_property.as_ptr(),
                    tip.map(|tip| tip.as_ptr()).unwrap_or(ptr::null()),
                    icon_property.as_ptr(),
                    icon.map(|icon| icon.as_ptr()).unwrap_or(ptr::null()),
                    ptr::null::<c_char>(),
                ) as *mut NautilusMenuItem
            }
        }
        None => unsafe {
            nautilus_menu_item_new(
                name.as_ptr(),
                label.as_ptr(),
                tip.map(|tip| tip.as_ptr()).unwrap_or(ptr::null()),
                icon.map(|icon| icon.as_ptr()).unwrap_or(ptr::null()),
            )
        },
    };

    if raw_menuitem.is_null() {
        None
    } else {
        Some(raw_menuitem)
    }
}
