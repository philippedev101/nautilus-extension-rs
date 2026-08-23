use super::*;

/// Provides custom context-menu items for selected files and backgrounds.
///
/// Method names mirror Nautilus' `MenuProvider` interface. Implement
/// [`MenuProvider::get_file_items`] for selections, and
/// [`MenuProvider::get_background_items`] for the current folder background.
///
/// # Example
///
/// ```no_run
/// use nautilus_extension::{FileInfo, MenuItem, MenuProvider};
///
/// struct Provider;
///
/// impl MenuProvider for Provider {
///     fn get_file_items(&self, files: &[FileInfo]) -> Vec<MenuItem> {
///         if files.is_empty() {
///             return Vec::new();
///         }
///
///         vec![MenuItem::new("Example::selected", "Example action").on_activate(|activation| {
///             for file in activation.files() {
///                 let _ = file.uri();
///             }
///         })]
///     }
/// }
/// ```
pub trait MenuProvider: Send + Sync {
    /// Returns context-menu items for the selected files.
    fn get_file_items(&self, _files: &[FileInfo]) -> Vec<MenuItem> {
        Vec::new()
    }

    /// Returns context-menu items with access to the live provider handle.
    fn get_file_items_full(
        &self,
        _provider: &MenuProviderHandle,
        files: &[FileInfo],
    ) -> Vec<MenuItem> {
        self.get_file_items(files)
    }

    /// Returns context-menu items for the current folder background.
    fn get_background_items(&self, _current_folder: &FileInfo) -> Vec<MenuItem> {
        Vec::new()
    }

    /// Returns background context-menu items with access to the live provider handle.
    fn get_background_items_full(
        &self,
        _provider: &MenuProviderHandle,
        current_folder: &FileInfo,
    ) -> Vec<MenuItem> {
        self.get_background_items(current_folder)
    }
}

/// Overrides the `NautilusMenuItemClass.activate` virtual method for a custom
/// menu-item subtype registered through `NautilusModule`.
pub trait MenuItemActivate: Send + Sync {
    /// Runs when Nautilus activates the custom menu item subtype.
    fn activate(&self, item: &MenuItemObject);
}

impl<F> MenuItemActivate for F
where
    F: Fn(&MenuItemObject) + Send + Sync,
{
    fn activate(&self, item: &MenuItemObject) {
        self(item);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Registered `NautilusMenuItem` subtype.
pub struct MenuItemType {
    raw: GType,
}

impl MenuItemType {
    /// # Safety
    ///
    /// `raw` must be a valid registered `NautilusMenuItem` subtype.
    pub unsafe fn from_raw(raw: GType) -> Option<MenuItemType> {
        if raw == 0 {
            None
        } else {
            Some(MenuItemType { raw })
        }
    }

    /// Returns the raw registered GType.
    pub fn raw(self) -> GType {
        self.raw
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Identifier returned by a GObject signal connection.
pub struct SignalHandlerId {
    raw: gulong,
}

impl SignalHandlerId {
    /// Creates a signal handler id from a nonzero raw GLib handler id.
    pub fn from_raw(raw: gulong) -> Option<SignalHandlerId> {
        if raw == 0 {
            None
        } else {
            Some(SignalHandlerId { raw })
        }
    }

    /// Returns the raw GLib signal handler id.
    pub fn raw(self) -> gulong {
        self.raw
    }
}

#[derive(Debug)]
/// Owned handle for the active Nautilus menu provider instance.
pub struct MenuProviderHandle {
    raw: *mut NautilusMenuProvider,
}

impl MenuProviderHandle {
    /// Returns the registered `NautilusMenuProvider` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_menu_provider_get_type() }
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer
    /// `NautilusMenuProvider` GObject pointer. On success, the returned wrapper
    /// owns that reference.
    pub unsafe fn from_raw_full(raw: *mut NautilusMenuProvider) -> Option<MenuProviderHandle> {
        if raw.is_null() {
            return None;
        }

        Some(MenuProviderHandle { raw })
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusMenuProvider` GObject pointer.
    /// This function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(raw: *mut NautilusMenuProvider) -> Option<MenuProviderHandle> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut GObject);
        }
        Some(MenuProviderHandle { raw })
    }

    /// Returns the wrapped raw `NautilusMenuProvider` pointer.
    pub fn raw(&self) -> *mut NautilusMenuProvider {
        self.raw
    }

    /// Returns the wrapped raw `NautilusMenuProvider` pointer.
    pub fn as_ptr(&self) -> *mut NautilusMenuProvider {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusMenuProvider {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    /// Emits Nautilus' `items-updated` signal for this provider.
    pub fn emit_items_updated_signal(&self) {
        unsafe {
            nautilus_menu_provider_emit_items_updated_signal(self.raw);
        }
    }

    /// Connects a callback to Nautilus' `items-updated` signal.
    pub fn connect_items_updated<F>(&self, callback: F) -> Option<SignalHandlerId>
    where
        F: Fn(&MenuProviderHandle) + 'static,
    {
        let signal_name = CString::new("items-updated").ok()?;
        let signal_data = Box::into_raw(Box::new(MenuProviderItemsUpdatedData {
            callback: Box::new(callback),
        }));

        let signal_id = unsafe {
            g_signal_connect_data(
                self.raw as *mut GObject,
                signal_name.as_ptr(),
                Some(std::mem::transmute::<
                    unsafe extern "C" fn(*mut NautilusMenuProvider, gpointer),
                    unsafe extern "C" fn(),
                >(menu_provider_items_updated_trampoline)),
                signal_data as gpointer,
                Some(destroy_menu_provider_items_updated_data),
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

    /// Disconnects a signal handler previously connected on this provider.
    pub fn disconnect_signal(&self, signal_id: SignalHandlerId) {
        unsafe {
            g_signal_handler_disconnect(self.raw as *mut GObject, signal_id.raw());
        }
    }

    /// Calls the provider interface for selected-file menu items.
    pub fn get_file_items(&self, files: &[FileInfo]) -> Vec<MenuItemObject> {
        let mut raw_files: *mut GList = ptr::null_mut();

        for file in files {
            unsafe {
                raw_files = g_list_append(raw_files, file.raw() as *mut c_void);
            }
        }

        let items = unsafe { nautilus_menu_provider_get_file_items(self.raw, raw_files) };

        unsafe {
            g_list_free(raw_files);
        }

        unsafe { MenuItemList::from_raw_full(items) }
            .map(|items| items.items())
            .unwrap_or_default()
    }

    /// Calls the provider interface for background menu items.
    pub fn get_background_items(&self, current_folder: &FileInfo) -> Vec<MenuItemObject> {
        let items =
            unsafe { nautilus_menu_provider_get_background_items(self.raw, current_folder.raw()) };

        unsafe { MenuItemList::from_raw_full(items) }
            .map(|items| items.items())
            .unwrap_or_default()
    }

    /// Calls the provider interface for selected-file menu items.
    ///
    /// This is the Rust-style alias for [`MenuProviderHandle::get_file_items`].
    pub fn file_items(&self, files: &[FileInfo]) -> Vec<MenuItemObject> {
        self.get_file_items(files)
    }

    /// Calls the provider interface for background menu items.
    ///
    /// This is the Rust-style alias for
    /// [`MenuProviderHandle::get_background_items`].
    pub fn background_items(&self, current_folder: &FileInfo) -> Vec<MenuItemObject> {
        self.get_background_items(current_folder)
    }
}

impl Clone for MenuProviderHandle {
    fn clone(&self) -> MenuProviderHandle {
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        MenuProviderHandle { raw: self.raw }
    }
}

impl Drop for MenuProviderHandle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut GObject);
            }
        }
    }
}

#[derive(Clone)]
/// A submenu attached to a [`MenuItem`].
pub struct Menu {
    pub(crate) menu_items: Vec<MenuItem>,
}

impl Menu {
    /// Creates a menu from owned items.
    pub fn new(menu_items: Vec<MenuItem>) -> Menu {
        Menu { menu_items }
    }

    /// Creates a menu by cloning a slice of items.
    pub fn from_slice(menu_items: &[MenuItem]) -> Menu {
        Menu {
            menu_items: menu_items.to_owned(),
        }
    }

    /// Returns the items in this menu.
    pub fn items(&self) -> &[MenuItem] {
        &self.menu_items
    }

    /// Builds the corresponding native `NautilusMenu` object.
    pub fn to_object(&self, target: &MenuActivationTarget) -> Option<MenuObject> {
        unsafe { MenuObject::from_raw_full(self.to_raw(target)) }
    }

    pub(crate) fn to_g_list(&self, target: &MenuActivationTarget) -> *mut GList {
        let mut raw_file_items: *mut GList = ptr::null_mut();

        for menu_item in &self.menu_items {
            if let Some(raw_menuitem) = menu_item.to_raw(target) {
                unsafe {
                    raw_file_items = g_list_append(raw_file_items, raw_menuitem as *mut c_void);
                }
            }
        }

        raw_file_items
    }

    pub(crate) fn to_raw(&self, target: &MenuActivationTarget) -> *mut NautilusMenu {
        let raw_menu = unsafe { nautilus_menu_new() };

        if raw_menu.is_null() {
            return raw_menu;
        }

        for menu_item in &self.menu_items {
            if let Some(raw_menuitem) = menu_item.to_raw(target) {
                unsafe {
                    nautilus_menu_append_item(raw_menu, raw_menuitem);
                    g_object_unref(raw_menuitem as *mut GObject);
                }
            }
        }

        raw_menu
    }
}
