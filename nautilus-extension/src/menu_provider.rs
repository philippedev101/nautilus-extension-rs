#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::glib_ffi::g_list_free;
use crate::glib_ffi::{g_list_append, gpointer, gulong, GList, GType};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gobject_ffi::g_object_new;
#[cfg(any(test, not(nautilus_extension_rs_skip_link)))]
use crate::gobject_ffi::GClosure;
use crate::gobject_ffi::{g_object_ref, g_object_unref, GObject};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gobject_ffi::{g_signal_connect_data, g_signal_handler_disconnect};
use crate::gobject_utils::{
    get_bool_property, get_object_property, get_string_property, set_bool_property,
    set_optional_string_property, set_string_property,
};
use crate::info_provider::FileInfo;
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::nautilus_ffi::{
    nautilus_menu_append_item, nautilus_menu_get_items, nautilus_menu_get_type,
    nautilus_menu_item_activate, nautilus_menu_item_get_type, nautilus_menu_item_list_free,
    nautilus_menu_item_new, nautilus_menu_item_set_submenu, nautilus_menu_new,
    nautilus_menu_provider_emit_items_updated_signal, nautilus_menu_provider_get_background_items,
    nautilus_menu_provider_get_file_items, nautilus_menu_provider_get_type,
};
use crate::nautilus_ffi::{
    NautilusFileInfo, NautilusMenu, NautilusMenuItem, NautilusMenuItemClass, NautilusMenuProvider,
    NautilusMenuProviderIface,
};
use crate::slot_allocator::{release_slot, reset_slots, take_next_slot};
use crate::translate::{file_info_vec_from_g_list, vec_from_g_list};
#[cfg(not(nautilus_extension_rs_skip_link))]
use libc::c_char;
use libc::c_void;
use std::borrow::Cow;
#[cfg(not(nautilus_extension_rs_skip_link))]
use std::ffi::CString;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

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
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusMenuProvider` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_menu_provider_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusMenuProvider` GType.
    pub fn type_() -> GType {
        0
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

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Emits Nautilus' `items-updated` signal for this provider.
    pub fn emit_items_updated_signal(&self) {
        unsafe {
            nautilus_menu_provider_emit_items_updated_signal(self.raw);
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Emits Nautilus' `items-updated` signal for this provider.
    pub fn emit_items_updated_signal(&self) {}

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Connects a callback to Nautilus' `items-updated` signal.
    pub fn connect_items_updated<F>(&self, _callback: F) -> Option<SignalHandlerId>
    where
        F: Fn(&MenuProviderHandle) + 'static,
    {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Disconnects a signal handler previously connected on this provider.
    pub fn disconnect_signal(&self, signal_id: SignalHandlerId) {
        unsafe {
            g_signal_handler_disconnect(self.raw as *mut GObject, signal_id.raw());
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Disconnects a signal handler previously connected on this provider.
    pub fn disconnect_signal(&self, _signal_id: SignalHandlerId) {}

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Calls the provider interface for selected-file menu items.
    pub fn get_file_items(&self, _files: &[FileInfo]) -> Vec<MenuItemObject> {
        Vec::new()
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Calls the provider interface for background menu items.
    pub fn get_background_items(&self, current_folder: &FileInfo) -> Vec<MenuItemObject> {
        let items =
            unsafe { nautilus_menu_provider_get_background_items(self.raw, current_folder.raw()) };

        unsafe { MenuItemList::from_raw_full(items) }
            .map(|items| items.items())
            .unwrap_or_default()
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Calls the provider interface for background menu items.
    pub fn get_background_items(&self, _current_folder: &FileInfo) -> Vec<MenuItemObject> {
        Vec::new()
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
    menu_items: Vec<MenuItem>,
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

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Builds the corresponding native `NautilusMenu` object.
    pub fn to_object(&self, target: &MenuActivationTarget) -> Option<MenuObject> {
        unsafe { MenuObject::from_raw_full(self.to_raw(target)) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Builds the corresponding native `NautilusMenu` object.
    pub fn to_object(&self, _target: &MenuActivationTarget) -> Option<MenuObject> {
        None
    }

    fn to_g_list(&self, target: &MenuActivationTarget) -> *mut GList {
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

    #[cfg(not(nautilus_extension_rs_skip_link))]
    fn to_raw(&self, target: &MenuActivationTarget) -> *mut NautilusMenu {
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

    #[cfg(nautilus_extension_rs_skip_link)]
    #[allow(dead_code)]
    fn to_raw(&self, _target: &MenuActivationTarget) -> *mut NautilusMenu {
        ptr::null_mut()
    }
}

#[derive(Debug)]
/// Owned reference to a `NautilusMenu` object.
pub struct MenuObject {
    raw: *mut NautilusMenu,
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
    item_type: Option<MenuItemType>,
    name: Cow<'static, str>,
    label: Cow<'static, str>,
    tip: Option<Cow<'static, str>>,
    icon: Option<Cow<'static, str>>,
    sensitive: bool,
    priority: Option<bool>,
    submenu: Option<Menu>,
    activate_fn: Option<Arc<dyn Fn(MenuActivation)>>,
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
    fn to_raw(&self, target: &MenuActivationTarget) -> Option<*mut NautilusMenuItem> {
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
    fn to_raw(&self, _target: &MenuActivationTarget) -> Option<*mut NautilusMenuItem> {
        None
    }
}

#[cfg(not(nautilus_extension_rs_skip_link))]
fn new_menu_item_raw(
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

#[derive(Debug)]
/// Owned reference to a `NautilusMenuItem` object.
pub struct MenuItemObject {
    raw: *mut NautilusMenuItem,
}

impl MenuItemObject {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusMenuItem` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_menu_item_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusMenuItem` GType.
    pub fn type_() -> GType {
        0
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Creates a native menu item.
    pub fn new<N, L>(_name: N, _label: L) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
    {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    #[deprecated(
        note = "Nautilus API 4.1 deprecates tip/icon constructor arguments; use new() for modern extensions"
    )]
    /// Creates a native menu item with deprecated tip and icon fields.
    pub fn new_full<N, L, T, I>(
        _name: N,
        _label: L,
        _tip: Option<T>,
        _icon: Option<I>,
    ) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
        T: AsRef<str>,
        I: AsRef<str>,
    {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Creates a native menu item using a custom item subtype.
    pub fn new_for_type<N, L>(
        _item_type: MenuItemType,
        _name: N,
        _label: L,
    ) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
    {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    #[deprecated(
        note = "Nautilus API 4.1 deprecates tip/icon constructor arguments; use new_for_type() for modern extensions"
    )]
    /// Creates a custom-subtype menu item with deprecated tip and icon fields.
    pub fn new_full_for_type<N, L, T, I>(
        _item_type: MenuItemType,
        _name: N,
        _label: L,
        _tip: Option<T>,
        _icon: Option<I>,
    ) -> Option<MenuItemObject>
    where
        N: AsRef<str>,
        L: AsRef<str>,
        T: AsRef<str>,
        I: AsRef<str>,
    {
        None
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

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Activates this native menu item.
    pub fn activate(&self) {
        unsafe {
            nautilus_menu_item_activate(self.raw);
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Activates this native menu item.
    pub fn activate(&self) {}

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Connects a callback to this item object's `activate` signal.
    pub fn connect_activate<F>(&self, _callback: F) -> Option<SignalHandlerId>
    where
        F: Fn(&MenuItemObject) + 'static,
    {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Disconnects a signal handler previously connected on this item.
    pub fn disconnect_signal(&self, signal_id: SignalHandlerId) {
        unsafe {
            g_signal_handler_disconnect(self.raw as *mut GObject, signal_id.raw());
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Disconnects a signal handler previously connected on this item.
    pub fn disconnect_signal(&self, _signal_id: SignalHandlerId) {}

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Attaches a submenu to this menu item.
    pub fn set_submenu(&self, submenu: &MenuObject) {
        unsafe {
            nautilus_menu_item_set_submenu(self.raw, submenu.raw());
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Attaches a submenu to this menu item.
    pub fn set_submenu(&self, _submenu: &MenuObject) {}

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Attaches `submenu` when present.
    ///
    /// Returns `false` for `None` because Nautilus API 4 does not expose a
    /// documented clear-submenu call.
    pub fn set_optional_submenu(&self, submenu: Option<&MenuObject>) -> bool {
        match submenu {
            Some(submenu) => {
                self.set_submenu(submenu);
                true
            }
            None => false,
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Attaches `submenu` when present.
    pub fn set_optional_submenu(&self, _submenu: Option<&MenuObject>) -> bool {
        false
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Attempts to clear the submenu.
    ///
    /// This returns `false` because Nautilus API 4 does not expose a documented
    /// clear-submenu call.
    pub fn clear_submenu(&self) -> bool {
        false
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Attempts to clear the submenu.
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

#[derive(Clone)]
/// Data available when a menu item is activated.
pub struct MenuActivation {
    target: MenuActivationTarget,
}

impl MenuActivation {
    /// Returns the selected files for a selected-file activation.
    pub fn files(&self) -> &[FileInfo] {
        match &self.target {
            MenuActivationTarget::Files(files) => files,
            MenuActivationTarget::Background(_) => &[],
        }
    }

    /// Returns the current folder for a background activation.
    pub fn current_folder(&self) -> Option<&FileInfo> {
        match &self.target {
            MenuActivationTarget::Files(_) => None,
            MenuActivationTarget::Background(folder) => Some(folder),
        }
    }

    /// Returns the activation target.
    pub fn target(&self) -> &MenuActivationTarget {
        &self.target
    }
}

#[derive(Clone)]
/// The Nautilus target that caused a menu item activation.
pub enum MenuActivationTarget {
    /// Activation from selected files.
    Files(Vec<FileInfo>),
    /// Activation from the current folder background.
    Background(FileInfo),
}

#[cfg(any(test, not(nautilus_extension_rs_skip_link)))]
#[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
struct MenuProviderItemsUpdatedData {
    callback: Box<dyn Fn(&MenuProviderHandle)>,
}

#[cfg(not(nautilus_extension_rs_skip_link))]
unsafe extern "C" fn menu_provider_items_updated_trampoline(
    provider: *mut NautilusMenuProvider,
    user_data: gpointer,
) {
    if user_data.is_null() {
        return;
    }

    let signal_data = unsafe { &*(user_data as *const MenuProviderItemsUpdatedData) };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(provider) = unsafe { MenuProviderHandle::from_raw_borrowed(provider) } {
            (signal_data.callback)(&provider);
        }
    }));
}

#[cfg(any(test, not(nautilus_extension_rs_skip_link)))]
unsafe extern "C" fn destroy_menu_provider_items_updated_data(
    user_data: gpointer,
    _closure: *mut GClosure,
) {
    if !user_data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(unsafe { Box::from_raw(user_data as *mut MenuProviderItemsUpdatedData) });
        }));
    }
}

#[cfg(any(test, not(nautilus_extension_rs_skip_link)))]
#[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
struct MenuItemObjectActivateData {
    callback: Box<dyn Fn(&MenuItemObject)>,
}

#[cfg(not(nautilus_extension_rs_skip_link))]
unsafe extern "C" fn menu_item_object_activate_trampoline(
    item: *mut NautilusMenuItem,
    user_data: gpointer,
) {
    if user_data.is_null() {
        return;
    }

    let signal_data = unsafe { &*(user_data as *const MenuItemObjectActivateData) };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Some(item) = unsafe { MenuItemObject::from_raw_borrowed(item) } {
            (signal_data.callback)(&item);
        }
    }));
}

#[cfg(any(test, not(nautilus_extension_rs_skip_link)))]
unsafe extern "C" fn destroy_menu_item_object_activate_data(
    user_data: gpointer,
    _closure: *mut GClosure,
) {
    if !user_data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(unsafe { Box::from_raw(user_data as *mut MenuItemObjectActivateData) });
        }));
    }
}

#[cfg(any(test, not(nautilus_extension_rs_skip_link)))]
#[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
struct ActivateData {
    activate_fn: Arc<dyn Fn(MenuActivation)>,
    target: MenuActivationTarget,
}

#[cfg(not(nautilus_extension_rs_skip_link))]
unsafe extern "C" fn menu_item_activate_trampoline(
    _item: *mut NautilusMenuItem,
    user_data: gpointer,
) {
    if user_data.is_null() {
        return;
    }

    let activate_data = unsafe { &*(user_data as *const ActivateData) };
    let activation = MenuActivation {
        target: activate_data.target.clone(),
    };

    let _ = catch_unwind(AssertUnwindSafe(|| {
        (activate_data.activate_fn)(activation);
    }));
}

#[cfg(any(test, not(nautilus_extension_rs_skip_link)))]
unsafe extern "C" fn destroy_activate_data(user_data: gpointer, _closure: *mut GClosure) {
    if !user_data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(unsafe { Box::from_raw(user_data as *mut ActivateData) });
        }));
    }
}

#[cfg(not(nautilus_extension_rs_skip_link))]
fn connect_activate_signal(
    raw_menuitem: *mut NautilusMenuItem,
    activate_fn: Arc<dyn Fn(MenuActivation)>,
    target: MenuActivationTarget,
) {
    let activate_name = match CString::new("activate") {
        Ok(activate_name) => activate_name,
        Err(_) => return,
    };

    let activate_data = Box::new(ActivateData {
        activate_fn,
        target,
    });

    let activate_data = Box::into_raw(activate_data);
    let signal_id = unsafe {
        g_signal_connect_data(
            raw_menuitem as *mut GObject,
            activate_name.as_ptr(),
            Some(std::mem::transmute::<
                unsafe extern "C" fn(*mut NautilusMenuItem, gpointer),
                unsafe extern "C" fn(),
            >(menu_item_activate_trampoline)),
            activate_data as gpointer,
            Some(destroy_activate_data),
            0,
        )
    };

    if signal_id == 0 {
        unsafe {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                drop(Box::from_raw(activate_data));
            }));
        }
    }
}

macro_rules! menu_item_class {
    ($class_init_fn:ident, $activate_fn:ident, $rust_activate:ident, $set_rust_activate:ident, $clear_rust_activate:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a GObject class initializer. Do not call directly.
        /// Use `NautilusModule.try_register_menu_item_type()` instead.
        unsafe extern "C" fn $class_init_fn(class: gpointer, _: gpointer) {
            if class.is_null() {
                return;
            }

            let item_class = class as *mut NautilusMenuItemClass;
            unsafe {
                (*item_class).activate = Some($activate_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as Nautilus' `MenuItemClass.activate`
        /// callback. Do not call directly.
        unsafe extern "C" fn $activate_fn(item: *mut NautilusMenuItem) {
            let rust_activate = $rust_activate
                .lock()
                .ok()
                .and_then(|activate| activate.clone());

            if let Some(activate) = rust_activate {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    if let Some(item) = unsafe { MenuItemObject::from_raw_borrowed(item) } {
                        activate.activate(&item);
                    }
                }));
            }
        }

        fn $set_rust_activate(activate: Box<dyn MenuItemActivate>) {
            if let Ok(mut current_activate) = $rust_activate.lock() {
                *current_activate = Some(Arc::from(activate));
            }
        }

        fn $clear_rust_activate() {
            if let Ok(mut activate) = $rust_activate.lock() {
                *activate = None;
            }
        }

        lazy_static! {
            static ref $rust_activate: Mutex<Option<Arc<dyn MenuItemActivate>>> = Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_MENU_ITEM_ACTIVATORS: usize = 10;

#[rustfmt::skip] menu_item_class!(menu_item_class_init_0, menu_item_activate_0, MENU_ITEM_ACTIVATE_0, set_menu_item_activate_0, clear_menu_item_activate_0);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_1, menu_item_activate_1, MENU_ITEM_ACTIVATE_1, set_menu_item_activate_1, clear_menu_item_activate_1);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_2, menu_item_activate_2, MENU_ITEM_ACTIVATE_2, set_menu_item_activate_2, clear_menu_item_activate_2);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_3, menu_item_activate_3, MENU_ITEM_ACTIVATE_3, set_menu_item_activate_3, clear_menu_item_activate_3);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_4, menu_item_activate_4, MENU_ITEM_ACTIVATE_4, set_menu_item_activate_4, clear_menu_item_activate_4);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_5, menu_item_activate_5, MENU_ITEM_ACTIVATE_5, set_menu_item_activate_5, clear_menu_item_activate_5);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_6, menu_item_activate_6, MENU_ITEM_ACTIVATE_6, set_menu_item_activate_6, clear_menu_item_activate_6);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_7, menu_item_activate_7, MENU_ITEM_ACTIVATE_7, set_menu_item_activate_7, clear_menu_item_activate_7);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_8, menu_item_activate_8, MENU_ITEM_ACTIVATE_8, set_menu_item_activate_8, clear_menu_item_activate_8);
#[rustfmt::skip] menu_item_class!(menu_item_class_init_9, menu_item_activate_9, MENU_ITEM_ACTIVATE_9, set_menu_item_activate_9, clear_menu_item_activate_9);

#[doc(hidden)]
pub fn menu_item_class_init_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        menu_item_class_init_0,
        menu_item_class_init_1,
        menu_item_class_init_2,
        menu_item_class_init_3,
        menu_item_class_init_4,
        menu_item_class_init_5,
        menu_item_class_init_6,
        menu_item_class_init_7,
        menu_item_class_init_8,
        menu_item_class_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_menu_item_activate_setters() -> Vec<fn(Box<dyn MenuItemActivate>)> {
    vec![
        set_menu_item_activate_0,
        set_menu_item_activate_1,
        set_menu_item_activate_2,
        set_menu_item_activate_3,
        set_menu_item_activate_4,
        set_menu_item_activate_5,
        set_menu_item_activate_6,
        set_menu_item_activate_7,
        set_menu_item_activate_8,
        set_menu_item_activate_9,
    ]
}

fn rust_menu_item_activate_clearers() -> Vec<fn()> {
    vec![
        clear_menu_item_activate_0,
        clear_menu_item_activate_1,
        clear_menu_item_activate_2,
        clear_menu_item_activate_3,
        clear_menu_item_activate_4,
        clear_menu_item_activate_5,
        clear_menu_item_activate_6,
        clear_menu_item_activate_7,
        clear_menu_item_activate_8,
        clear_menu_item_activate_9,
    ]
}

static RESERVED_MENU_ITEM_CLASS_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_menu_item_class_index() -> Option<usize> {
    take_next_slot(&RESERVED_MENU_ITEM_CLASS_SLOTS, MAX_MENU_ITEM_ACTIVATORS)
}

pub(crate) fn release_menu_item_class_index(index: usize) {
    if let Some(clear_activate) = rust_menu_item_activate_clearers().get(index) {
        clear_activate();
    }

    release_slot(&RESERVED_MENU_ITEM_CLASS_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn menu_item_activate_slot_is_set(index: usize) -> bool {
    match index {
        0 => MENU_ITEM_ACTIVATE_0
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        1 => MENU_ITEM_ACTIVATE_1
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        2 => MENU_ITEM_ACTIVATE_2
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        3 => MENU_ITEM_ACTIVATE_3
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        4 => MENU_ITEM_ACTIVATE_4
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        5 => MENU_ITEM_ACTIVATE_5
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        6 => MENU_ITEM_ACTIVATE_6
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        7 => MENU_ITEM_ACTIVATE_7
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        8 => MENU_ITEM_ACTIVATE_8
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        9 => MENU_ITEM_ACTIVATE_9
            .lock()
            .map(|activate| activate.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_menu_item_activate_state() {
    reset_slots(&RESERVED_MENU_ITEM_CLASS_SLOTS);

    for clear_activate in rust_menu_item_activate_clearers() {
        clear_activate();
    }
}

macro_rules! menu_provider_iface {
    ($iface_init_fn:ident, $get_file_items_fn:ident, $get_background_items_fn:ident, $rust_provider:ident, $set_rust_provider:ident, $clear_rust_provider:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_menu_provider()` instead.
        unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusMenuProviderIface;
            unsafe {
                (*iface_struct).get_file_items = Some($get_file_items_fn);
                (*iface_struct).get_background_items = Some($get_background_items_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_file_items_fn(
            provider: *mut NautilusMenuProvider,
            files: *mut GList,
        ) -> *mut GList {
            catch_unwind(AssertUnwindSafe(|| {
                let files_vec = file_info_vec_from_g_list(files);
                let target = MenuActivationTarget::Files(files_vec.clone());
                let handle = unsafe { MenuProviderHandle::from_raw_borrowed(provider) };
                let rust_provider = $rust_provider
                    .lock()
                    .ok()
                    .and_then(|provider| provider.clone());

                let file_items: Vec<MenuItem> = match rust_provider {
                    Some(provider) => match handle.as_ref() {
                        Some(handle) => catch_unwind(AssertUnwindSafe(|| {
                            provider.get_file_items_full(handle, &files_vec)
                        }))
                        .unwrap_or_else(|_| Vec::new()),
                        None => {
                            catch_unwind(AssertUnwindSafe(|| provider.get_file_items(&files_vec)))
                                .unwrap_or_else(|_| Vec::new())
                        }
                    },
                    None => Vec::new(),
                };

                Menu {
                    menu_items: file_items,
                }
                .to_g_list(&target)
            }))
            .unwrap_or(ptr::null_mut())
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_background_items_fn(
            provider: *mut NautilusMenuProvider,
            current_folder: *mut NautilusFileInfo,
        ) -> *mut GList {
            catch_unwind(AssertUnwindSafe(|| {
                let current_folder = match unsafe { FileInfo::from_raw_borrowed(current_folder) } {
                    Some(current_folder) => current_folder,
                    None => return ptr::null_mut(),
                };
                let target = MenuActivationTarget::Background(current_folder.clone());
                let handle = unsafe { MenuProviderHandle::from_raw_borrowed(provider) };
                let rust_provider = $rust_provider
                    .lock()
                    .ok()
                    .and_then(|provider| provider.clone());

                let file_items: Vec<MenuItem> = match rust_provider {
                    Some(provider) => match handle.as_ref() {
                        Some(handle) => catch_unwind(AssertUnwindSafe(|| {
                            provider.get_background_items_full(handle, &current_folder)
                        }))
                        .unwrap_or_else(|_| Vec::new()),
                        None => catch_unwind(AssertUnwindSafe(|| {
                            provider.get_background_items(&current_folder)
                        }))
                        .unwrap_or_else(|_| Vec::new()),
                    },
                    None => Vec::new(),
                };

                Menu {
                    menu_items: file_items,
                }
                .to_g_list(&target)
            }))
            .unwrap_or(ptr::null_mut())
        }

        fn $set_rust_provider(menu_provider: Box<dyn MenuProvider>) {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = Some(Arc::from(menu_provider));
            }
        }

        fn $clear_rust_provider() {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = None;
            }
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Arc<dyn MenuProvider>>> = Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_MENU_PROVIDERS: usize = 10;

#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_0, menu_provider_get_file_items_0, menu_provider_get_background_items_0, MENU_PROVIDER_0, set_menu_provider_0, clear_menu_provider_0);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_1, menu_provider_get_file_items_1, menu_provider_get_background_items_1, MENU_PROVIDER_1, set_menu_provider_1, clear_menu_provider_1);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_2, menu_provider_get_file_items_2, menu_provider_get_background_items_2, MENU_PROVIDER_2, set_menu_provider_2, clear_menu_provider_2);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_3, menu_provider_get_file_items_3, menu_provider_get_background_items_3, MENU_PROVIDER_3, set_menu_provider_3, clear_menu_provider_3);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_4, menu_provider_get_file_items_4, menu_provider_get_background_items_4, MENU_PROVIDER_4, set_menu_provider_4, clear_menu_provider_4);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_5, menu_provider_get_file_items_5, menu_provider_get_background_items_5, MENU_PROVIDER_5, set_menu_provider_5, clear_menu_provider_5);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_6, menu_provider_get_file_items_6, menu_provider_get_background_items_6, MENU_PROVIDER_6, set_menu_provider_6, clear_menu_provider_6);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_7, menu_provider_get_file_items_7, menu_provider_get_background_items_7, MENU_PROVIDER_7, set_menu_provider_7, clear_menu_provider_7);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_8, menu_provider_get_file_items_8, menu_provider_get_background_items_8, MENU_PROVIDER_8, set_menu_provider_8, clear_menu_provider_8);
#[rustfmt::skip] menu_provider_iface!(menu_provider_iface_init_9, menu_provider_get_file_items_9, menu_provider_get_background_items_9, MENU_PROVIDER_9, set_menu_provider_9, clear_menu_provider_9);

#[doc(hidden)]
pub fn menu_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        menu_provider_iface_init_0,
        menu_provider_iface_init_1,
        menu_provider_iface_init_2,
        menu_provider_iface_init_3,
        menu_provider_iface_init_4,
        menu_provider_iface_init_5,
        menu_provider_iface_init_6,
        menu_provider_iface_init_7,
        menu_provider_iface_init_8,
        menu_provider_iface_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_menu_provider_setters() -> Vec<fn(Box<dyn MenuProvider>)> {
    vec![
        set_menu_provider_0,
        set_menu_provider_1,
        set_menu_provider_2,
        set_menu_provider_3,
        set_menu_provider_4,
        set_menu_provider_5,
        set_menu_provider_6,
        set_menu_provider_7,
        set_menu_provider_8,
        set_menu_provider_9,
    ]
}

fn rust_menu_provider_clearers() -> Vec<fn()> {
    vec![
        clear_menu_provider_0,
        clear_menu_provider_1,
        clear_menu_provider_2,
        clear_menu_provider_3,
        clear_menu_provider_4,
        clear_menu_provider_5,
        clear_menu_provider_6,
        clear_menu_provider_7,
        clear_menu_provider_8,
        clear_menu_provider_9,
    ]
}

static RESERVED_MENU_PROVIDER_IFACE_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_menu_provider_iface_index() -> Option<usize> {
    take_next_slot(&RESERVED_MENU_PROVIDER_IFACE_SLOTS, MAX_MENU_PROVIDERS)
}

pub(crate) fn release_menu_provider_iface_index(index: usize) {
    if let Some(clear_provider) = rust_menu_provider_clearers().get(index) {
        clear_provider();
    }

    release_slot(&RESERVED_MENU_PROVIDER_IFACE_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn menu_provider_slot_is_set(index: usize) -> bool {
    match index {
        0 => MENU_PROVIDER_0
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        1 => MENU_PROVIDER_1
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        2 => MENU_PROVIDER_2
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        3 => MENU_PROVIDER_3
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        4 => MENU_PROVIDER_4
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        5 => MENU_PROVIDER_5
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        6 => MENU_PROVIDER_6
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        7 => MENU_PROVIDER_7
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        8 => MENU_PROVIDER_8
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        9 => MENU_PROVIDER_9
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_menu_provider_state() {
    reset_slots(&RESERVED_MENU_PROVIDER_IFACE_SLOTS);
    for clear_provider in rust_menu_provider_clearers() {
        clear_provider();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    static MENU_DESTROY_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
    static MENU_GET_FILE_ITEMS_CALLS: AtomicUsize = AtomicUsize::new(0);

    struct PanickingMenuDropPayload;

    impl Drop for PanickingMenuDropPayload {
        fn drop(&mut self) {
            MENU_DESTROY_DROP_CALLS.fetch_add(1, Ordering::SeqCst);
            panic!("menu destroy callback payload drop panic");
        }
    }

    struct RoutedMenuProvider;

    impl MenuProvider for RoutedMenuProvider {
        fn get_file_items(&self, files: &[FileInfo]) -> Vec<MenuItem> {
            assert!(files.is_empty());
            MENU_GET_FILE_ITEMS_CALLS.fetch_add(1, Ordering::SeqCst);
            Vec::new()
        }
    }

    struct PanickingMenuProvider;

    impl MenuProvider for PanickingMenuProvider {
        fn get_file_items(&self, _files: &[FileInfo]) -> Vec<MenuItem> {
            panic!("menu provider callback panic");
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    struct PanickingBackgroundMenuProvider;

    #[cfg(not(nautilus_extension_rs_skip_link))]
    impl MenuProvider for PanickingBackgroundMenuProvider {
        fn get_background_items(&self, _current_folder: &FileInfo) -> Vec<MenuItem> {
            panic!("menu provider background callback panic");
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    struct PanickingMenuItemActivate;

    #[cfg(not(nautilus_extension_rs_skip_link))]
    impl MenuItemActivate for PanickingMenuItemActivate {
        fn activate(&self, _item: &MenuItemObject) {
            panic!("menu item activate callback panic");
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    fn opaque_native_file_info() -> FileInfo {
        let raw = unsafe {
            crate::gobject_ffi::g_object_new(
                crate::gobject_ffi::G_TYPE_OBJECT,
                ptr::null::<c_char>(),
            )
        };

        unsafe { FileInfo::from_raw_full(raw as *mut NautilusFileInfo) }
            .expect("GObject should be constructible for opaque native FileInfo tests")
    }

    #[test]
    fn menu_from_slice_clones_items() {
        let item = MenuItem::new("Example::item", "Example Item");
        let menu = Menu::from_slice(&[item]);

        assert_eq!(menu.menu_items.len(), 1);
        assert_eq!(menu.menu_items[0].name.as_ref(), "Example::item");
        assert_eq!(menu.menu_items[0].label.as_ref(), "Example Item");
    }

    #[test]
    #[allow(deprecated)]
    fn menu_item_builder_keeps_optional_state() {
        let item_type = unsafe { MenuItemType::from_raw(77) }.unwrap();
        let submenu = Menu::new(vec![MenuItem::new("Example::child", "Child")]);
        let item = MenuItem::new("Example::parent", "Parent")
            .with_menu_item_type(item_type)
            .with_tip("Tip")
            .with_icon("folder")
            .sensitive(false)
            .priority(false)
            .with_submenu(submenu)
            .on_activate(|_| {});

        assert_eq!(item.name.as_ref(), "Example::parent");
        assert_eq!(item.label.as_ref(), "Parent");
        assert_eq!(item.item_type, Some(item_type));
        assert_eq!(item.item_type(), Some(item_type));
        assert_eq!(item.tip.as_ref().map(|tip| tip.as_ref()), Some("Tip"));
        assert_eq!(item.icon.as_ref().map(|icon| icon.as_ref()), Some("folder"));
        assert!(!item.sensitive);
        assert_eq!(item.priority, Some(false));
        assert_eq!(
            item.submenu.as_ref().map(|menu| menu.menu_items.len()),
            Some(1)
        );
        assert!(item.activate_fn.is_some());
    }

    #[test]
    fn menu_item_type_rejects_zero_and_preserves_raw_gtype() {
        assert!(unsafe { MenuItemType::from_raw(0) }.is_none());

        let item_type = unsafe { MenuItemType::from_raw(77) }.unwrap();

        assert_eq!(item_type.raw(), 77);
    }

    #[test]
    fn menu_item_activate_trait_supports_closures() {
        let calls = AtomicUsize::new(0);
        let activate = |_: &MenuItemObject| {
            calls.fetch_add(1, Ordering::SeqCst);
        };
        let item = MenuItemObject {
            raw: ptr::null_mut(),
        };

        activate.activate(&item);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn menu_item_class_init_installs_activate_vfunc() {
        let mut class: NautilusMenuItemClass = unsafe { std::mem::zeroed() };

        unsafe {
            menu_item_class_init_0(
                &mut class as *mut NautilusMenuItemClass as gpointer,
                ptr::null_mut(),
            );
        }

        assert!(class.activate.is_some());
    }

    #[test]
    fn menu_object_wrappers_reject_null_raw_pointers() {
        assert!(unsafe { MenuObject::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { MenuObject::from_raw_borrowed(ptr::null_mut()) }.is_none());
        assert!(unsafe { MenuItemObject::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { MenuItemObject::from_raw_borrowed(ptr::null_mut()) }.is_none());
        assert!(unsafe { MenuItemList::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { MenuProviderHandle::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { MenuProviderHandle::from_raw_borrowed(ptr::null_mut()) }.is_none());
    }

    #[test]
    fn signal_handler_id_rejects_zero_and_preserves_raw_value() {
        assert_eq!(SignalHandlerId::from_raw(0), None);

        let signal_id = SignalHandlerId::from_raw(7).unwrap();
        assert_eq!(signal_id.raw(), 7);
    }

    #[test]
    fn menu_provider_iface_trampoline_routes_file_items_to_registered_impl() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_menu_provider_state();
        MENU_GET_FILE_ITEMS_CALLS.store(0, Ordering::SeqCst);

        set_menu_provider_0(Box::new(RoutedMenuProvider));

        let mut iface: NautilusMenuProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            menu_provider_iface_init_0(
                &mut iface as *mut NautilusMenuProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let items = unsafe { (iface.get_file_items.unwrap())(ptr::null_mut(), ptr::null_mut()) };

        assert!(items.is_null());
        assert_eq!(MENU_GET_FILE_ITEMS_CALLS.load(Ordering::SeqCst), 1);
        assert!(iface.get_background_items.is_some());

        reset_menu_provider_state();
    }

    #[test]
    fn menu_provider_iface_trampoline_catches_file_item_provider_panics() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_menu_provider_state();

        set_menu_provider_0(Box::new(PanickingMenuProvider));

        let mut iface: NautilusMenuProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            menu_provider_iface_init_0(
                &mut iface as *mut NautilusMenuProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let result = std::panic::catch_unwind(|| unsafe {
            (iface.get_file_items.unwrap())(ptr::null_mut(), ptr::null_mut())
        });

        assert_eq!(result.unwrap(), ptr::null_mut());

        reset_menu_provider_state();
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn menu_provider_iface_trampoline_catches_background_provider_panics() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_menu_provider_state();

        set_menu_provider_0(Box::new(PanickingBackgroundMenuProvider));

        let mut iface: NautilusMenuProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            menu_provider_iface_init_0(
                &mut iface as *mut NautilusMenuProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let folder = opaque_native_file_info();
        let result = std::panic::catch_unwind(|| unsafe {
            (iface.get_background_items.unwrap())(ptr::null_mut(), folder.raw())
        });

        assert_eq!(result.unwrap(), ptr::null_mut());

        reset_menu_provider_state();
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn documented_type_accessors_are_inert_in_no_link_mode() {
        assert_eq!(MenuObject::type_(), 0);
        assert_eq!(MenuItemObject::type_(), 0);
        assert_eq!(MenuProviderHandle::type_(), 0);
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn optional_submenu_assignment_is_inert_in_no_link_mode() {
        let item = MenuItemObject {
            raw: ptr::null_mut(),
        };
        let submenu = MenuObject {
            raw: ptr::null_mut(),
        };

        assert!(!item.set_optional_submenu(Some(&submenu)));
        assert!(!item.set_optional_submenu(None));
        assert!(!item.clear_submenu());
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn documented_type_accessors_return_registered_gtypes() {
        assert_ne!(MenuObject::type_(), 0);
        assert_ne!(MenuItemObject::type_(), 0);
        assert_ne!(MenuProviderHandle::type_(), 0);
    }

    #[test]
    fn signal_destroy_callbacks_catch_panicking_payload_drops() {
        MENU_DESTROY_DROP_CALLS.store(0, Ordering::SeqCst);

        let payload = PanickingMenuDropPayload;
        let provider_data = Box::new(MenuProviderItemsUpdatedData {
            callback: Box::new(move |_| {
                let _ = &payload;
            }),
        });
        unsafe {
            destroy_menu_provider_items_updated_data(
                Box::into_raw(provider_data) as gpointer,
                ptr::null_mut(),
            );
        }

        let payload = PanickingMenuDropPayload;
        let item_data = Box::new(MenuItemObjectActivateData {
            callback: Box::new(move |_| {
                let _ = &payload;
            }),
        });
        unsafe {
            destroy_menu_item_object_activate_data(
                Box::into_raw(item_data) as gpointer,
                ptr::null_mut(),
            );
        }

        let payload = PanickingMenuDropPayload;
        let activate_data = Box::new(ActivateData {
            activate_fn: Arc::new(move |_| {
                let _ = &payload;
            }),
            target: MenuActivationTarget::Files(Vec::new()),
        });
        unsafe {
            destroy_activate_data(Box::into_raw(activate_data) as gpointer, ptr::null_mut());
        }

        assert_eq!(MENU_DESTROY_DROP_CALLS.load(Ordering::SeqCst), 3);
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn signal_trampolines_catch_panicking_callbacks() {
        let file = opaque_native_file_info();
        let item = MenuItemObject::new("RustValidation::signal", "Signal").unwrap();

        let provider_data = Box::new(MenuProviderItemsUpdatedData {
            callback: Box::new(|_| panic!("items-updated signal callback panic")),
        });
        let provider_data = Box::into_raw(provider_data);
        let result = catch_unwind(AssertUnwindSafe(|| unsafe {
            menu_provider_items_updated_trampoline(
                item.raw() as *mut NautilusMenuProvider,
                provider_data as gpointer,
            );
        }));
        assert!(result.is_ok());
        unsafe {
            drop(Box::from_raw(provider_data));
        }

        let item_data = Box::new(MenuItemObjectActivateData {
            callback: Box::new(|_| panic!("menu item object activate callback panic")),
        });
        let item_data = Box::into_raw(item_data);
        let result = catch_unwind(AssertUnwindSafe(|| unsafe {
            menu_item_object_activate_trampoline(item.raw(), item_data as gpointer);
        }));
        assert!(result.is_ok());
        unsafe {
            drop(Box::from_raw(item_data));
        }

        let activate_data = Box::new(ActivateData {
            activate_fn: Arc::new(|_| panic!("menu activation callback panic")),
            target: MenuActivationTarget::Background(file),
        });
        let activate_data = Box::into_raw(activate_data);
        let result = catch_unwind(AssertUnwindSafe(|| unsafe {
            menu_item_activate_trampoline(item.raw(), activate_data as gpointer);
        }));
        assert!(result.is_ok());
        unsafe {
            drop(Box::from_raw(activate_data));
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn menu_item_activate_vfunc_catches_provider_panics() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_menu_item_activate_state();

        set_menu_item_activate_0(Box::new(PanickingMenuItemActivate));
        let item = MenuItemObject::new("RustValidation::activate", "Activate").unwrap();

        let result = std::panic::catch_unwind(|| unsafe {
            menu_item_activate_0(item.raw());
        });

        assert!(result.is_ok());

        reset_menu_item_activate_state();
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn set_submenu_accepts_borrowed_submenu_wrapper() {
        let item = MenuItemObject::new("Example::parent", "Parent").unwrap();
        let submenu = MenuObject::new().unwrap();

        item.set_submenu(&submenu);

        assert!(item.submenu().is_some());
        assert!(submenu.get_items().is_empty());
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn optional_submenu_assignment_can_attach_submenus() {
        let item = MenuItemObject::new("Example::parent", "Parent").unwrap();
        let submenu = MenuObject::new().unwrap();

        assert!(item.set_optional_submenu(Some(&submenu)));
        assert!(item.submenu().is_some());

        assert!(item.set_optional_submenu(Some(&submenu)));
        assert!(item.submenu().is_some());

        assert!(!item.set_optional_submenu(None));
        assert!(!item.clear_submenu());
    }

    #[test]
    fn reset_menu_provider_state_allows_slot_reuse() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_menu_provider_state();

        for expected_index in 0..MAX_MENU_PROVIDERS {
            assert_eq!(take_next_menu_provider_iface_index(), Some(expected_index));
        }

        assert_eq!(take_next_menu_provider_iface_index(), None);

        reset_menu_provider_state();

        assert_eq!(take_next_menu_provider_iface_index(), Some(0));

        reset_menu_provider_state();
    }

    #[test]
    fn reset_menu_item_activate_state_allows_slot_reuse() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_menu_item_activate_state();

        for expected_index in 0..MAX_MENU_ITEM_ACTIVATORS {
            assert_eq!(take_next_menu_item_class_index(), Some(expected_index));
        }

        assert_eq!(take_next_menu_item_class_index(), None);

        reset_menu_item_activate_state();

        assert_eq!(take_next_menu_item_class_index(), Some(0));

        reset_menu_item_activate_state();
    }

    #[test]
    fn menu_item_activate_slots_reuse_out_of_order_releases() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_menu_item_activate_state();

        let first = take_next_menu_item_class_index().unwrap();
        let second = take_next_menu_item_class_index().unwrap();

        assert_eq!(first, 0);
        assert_eq!(second, 1);

        release_menu_item_class_index(first);

        let reused = take_next_menu_item_class_index().unwrap();
        assert_eq!(reused, first);

        release_menu_item_class_index(second);
        release_menu_item_class_index(reused);

        assert_eq!(take_next_menu_item_class_index(), Some(0));

        reset_menu_item_activate_state();
    }
}
