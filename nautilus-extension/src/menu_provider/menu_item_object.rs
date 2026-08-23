use super::*;

#[derive(Debug)]
/// Owned reference to a `NautilusMenuItem` object.
pub struct MenuItemObject {
    pub(crate) raw: *mut NautilusMenuItem,
}

impl MenuItemObject {
    /// Returns the registered `NautilusMenuItem` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_menu_item_get_type() }
    }

    /// Creates a native menu item.
    pub fn new<N, L>(name: N, label: L) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
    {
        #[allow(deprecated)]
        {
            MenuItemObject::new_full(name, label, None::<&str>, None::<&str>)
        }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates tip/icon constructor arguments; use new() for modern extensions"
    )]
    /// Creates a native menu item with deprecated tip and icon fields.
    pub fn new_full<N, L, T, I>(
        name: N,
        label: L,
        tip: Option<T>,
        icon: Option<I>,
    ) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
        T: AsRef<str>,
        I: AsRef<str>,
    {
        let name = CString::new(name.as_ref()).ok()?;
        let label = CString::new(label.as_ref()).ok()?;
        let tip = tip.and_then(|tip| CString::new(tip.as_ref()).ok());
        let icon = icon.and_then(|icon| CString::new(icon.as_ref()).ok());

        unsafe {
            MenuItemObject::from_raw_full(new_menu_item_raw(
                None,
                &name,
                &label,
                tip.as_ref(),
                icon.as_ref(),
            )?)
        }
    }

    /// Creates a native menu item using a custom item subtype.
    pub fn new_for_type<N, L>(item_type: MenuItemType, name: N, label: L) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
    {
        #[allow(deprecated)]
        {
            MenuItemObject::new_full_for_type(item_type, name, label, None::<&str>, None::<&str>)
        }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates tip/icon constructor arguments; use new_for_type() for modern extensions"
    )]
    /// Creates a custom-subtype menu item with deprecated tip and icon fields.
    pub fn new_full_for_type<N, L, T, I>(
        item_type: MenuItemType,
        name: N,
        label: L,
        tip: Option<T>,
        icon: Option<I>,
    ) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
        T: AsRef<str>,
        I: AsRef<str>,
    {
        let name = CString::new(name.as_ref()).ok()?;
        let label = CString::new(label.as_ref()).ok()?;
        let tip = tip.and_then(|tip| CString::new(tip.as_ref()).ok());
        let icon = icon.and_then(|icon| CString::new(icon.as_ref()).ok());

        unsafe {
            MenuItemObject::from_raw_full(new_menu_item_raw(
                Some(item_type),
                &name,
                &label,
                tip.as_ref(),
                icon.as_ref(),
            )?)
        }
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusMenuItem` GObject pointer. This
    /// function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(raw: *mut NautilusMenuItem) -> Option<MenuItemObject> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut GObject);
        }
        Some(MenuItemObject { raw })
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer `NautilusMenuItem`
    /// GObject pointer. On success, the returned wrapper owns that reference.
    pub unsafe fn from_raw_full(raw: *mut NautilusMenuItem) -> Option<MenuItemObject> {
        if raw.is_null() {
            return None;
        }

        Some(MenuItemObject { raw })
    }

    /// Returns the wrapped raw `NautilusMenuItem` pointer.
    pub fn raw(&self) -> *mut NautilusMenuItem {
        self.raw
    }

    /// Returns the wrapped raw `NautilusMenuItem` pointer.
    pub fn as_ptr(&self) -> *mut NautilusMenuItem {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusMenuItem {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    /// Activates this native menu item.
    pub fn activate(&self) {
        unsafe {
            nautilus_menu_item_activate(self.raw);
        }
    }

    /// Connects a callback to this item object's `activate` signal.
    pub fn connect_activate<F>(&self, callback: F) -> Option<SignalHandlerId>
    where
        F: Fn(&MenuItemObject) + 'static,
    {
        let signal_name = CString::new("activate").ok()?;
        let signal_data = Box::into_raw(Box::new(MenuItemObjectActivateData {
            callback: Box::new(callback),
        }));

        let signal_id = unsafe {
            g_signal_connect_data(
                self.raw as *mut GObject,
                signal_name.as_ptr(),
                Some(std::mem::transmute::<
                    unsafe extern "C" fn(*mut NautilusMenuItem, gpointer),
                    unsafe extern "C" fn(),
                >(menu_item_object_activate_trampoline)),
                signal_data as gpointer,
                Some(destroy_menu_item_object_activate_data),
                0,
            )
        };

        match SignalHandlerId::from_raw(signal_id) {
            Some(signal_id) => Some(signal_id),
            None => {
                unsafe {
                    drop(Box::from_raw(signal_data));
                }
                None
            }
        }
    }

    /// Disconnects a signal handler previously connected on this item.
    pub fn disconnect_signal(&self, signal_id: SignalHandlerId) {
        unsafe {
            g_signal_handler_disconnect(self.raw as *mut GObject, signal_id.raw());
        }
    }

    /// Attaches a submenu to this menu item.
    pub fn set_submenu(&self, submenu: &MenuObject) {
        unsafe {
            nautilus_menu_item_set_submenu(self.raw, submenu.raw());
        }
    }

    /// Attaches `submenu` when present.
    ///
    /// Returns `false` for `None` because Nautilus API 4 does not expose a
    /// documented clear-submenu call.
    pub fn set_optional_submenu(&self, submenu: Option<&MenuObject>) -> bool {
        let Some(submenu) = submenu else {
            return false;
        };

        if !NATIVE_API_AVAILABLE {
            return false;
        }

        self.set_submenu(submenu);
        true
    }

    /// Attempts to clear the submenu.
    ///
    /// This returns `false` because Nautilus API 4 does not expose a documented
    /// clear-submenu call.
    pub fn clear_submenu(&self) -> bool {
        false
    }

    /// Returns the unique menu item name.
    pub fn name(&self) -> Option<String> {
        unsafe { get_string_property(self.raw as *mut GObject, "name") }
    }

    /// Returns the user-visible menu item label.
    pub fn label(&self) -> Option<String> {
        unsafe { get_string_property(self.raw as *mut GObject, "label") }
    }

    /// Sets the user-visible menu item label.
    pub fn set_label(&self, label: &str) -> bool {
        unsafe { set_string_property(self.raw as *mut GObject, "label", label) }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item tips; this is kept for compatibility"
    )]
    /// Returns the deprecated menu item tip.
    pub fn tip(&self) -> Option<String> {
        unsafe { get_string_property(self.raw as *mut GObject, "tip") }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item tips; this is kept for compatibility"
    )]
    /// Sets the deprecated menu item tip.
    pub fn set_tip(&self, tip: Option<&str>) -> bool {
        unsafe { set_optional_string_property(self.raw as *mut GObject, "tip", tip) }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item icons; this is kept for compatibility"
    )]
    /// Returns the deprecated menu item icon name.
    pub fn icon(&self) -> Option<String> {
        unsafe { get_string_property(self.raw as *mut GObject, "icon") }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates menu item icons; this is kept for compatibility"
    )]
    /// Sets the deprecated menu item icon name.
    pub fn set_icon(&self, icon: Option<&str>) -> bool {
        unsafe { set_optional_string_property(self.raw as *mut GObject, "icon", icon) }
    }

    /// Returns whether the item is sensitive.
    pub fn sensitive(&self) -> bool {
        unsafe { get_bool_property(self.raw as *mut GObject, "sensitive") }
    }

    /// Sets whether the item is sensitive.
    pub fn set_sensitive(&self, sensitive: bool) -> bool {
        unsafe { set_bool_property(self.raw as *mut GObject, "sensitive", sensitive) }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates the menu item priority property; this is kept for compatibility"
    )]
    /// Returns the deprecated priority flag.
    pub fn priority(&self) -> bool {
        unsafe { get_bool_property(self.raw as *mut GObject, "priority") }
    }

    #[deprecated(
        note = "Nautilus API 4.1 deprecates the menu item priority property; this is kept for compatibility"
    )]
    /// Sets the deprecated priority flag.
    pub fn set_priority(&self, priority: bool) -> bool {
        unsafe { set_bool_property(self.raw as *mut GObject, "priority", priority) }
    }

    /// Returns the attached submenu, if any.
    pub fn submenu(&self) -> Option<MenuObject> {
        unsafe {
            get_object_property(self.raw as *mut GObject, "menu")
                .and_then(|raw| MenuObject::from_raw_full(raw))
        }
    }
}

impl Clone for MenuItemObject {
    fn clone(&self) -> MenuItemObject {
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        MenuItemObject { raw: self.raw }
    }
}

impl Drop for MenuItemObject {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut GObject);
            }
        }
    }
}
