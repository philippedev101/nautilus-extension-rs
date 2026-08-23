use super::*;

#[derive(Clone)]
/// Data available when a menu item is activated.
pub struct MenuActivation {
    pub(crate) target: MenuActivationTarget,
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

pub(crate) struct MenuProviderItemsUpdatedData {
    pub(crate) callback: Box<dyn Fn(&MenuProviderHandle)>,
}

pub(crate) unsafe extern "C" fn menu_provider_items_updated_trampoline(
    provider: *mut NautilusMenuProvider,
    user_data: gpointer,
) {
    if user_data.is_null() {
        return;
    }

    // SAFETY: the payload pointer is the one this module boxed when the callback was
    // registered.
    let signal_data = unsafe { &*(user_data as *const MenuProviderItemsUpdatedData) };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Nautilus owns this pointer for the duration of the call, and
        // `from_raw_borrowed` rejects null and takes its own reference.
        if let Some(provider) = unsafe { MenuProviderHandle::from_raw_borrowed(provider) } {
            (signal_data.callback)(&provider);
        }
    }));
}

pub(crate) unsafe extern "C" fn destroy_menu_provider_items_updated_data(
    user_data: gpointer,
    _closure: *mut GClosure,
) {
    if !user_data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: the pointer is the one this module boxed for the callback and is
            // dropped exactly once.
            drop(unsafe { Box::from_raw(user_data as *mut MenuProviderItemsUpdatedData) });
        }));
    }
}

pub(crate) struct MenuItemObjectActivateData {
    pub(crate) callback: Box<dyn Fn(&MenuItemObject)>,
}

pub(crate) unsafe extern "C" fn menu_item_object_activate_trampoline(
    item: *mut NautilusMenuItem,
    user_data: gpointer,
) {
    if user_data.is_null() {
        return;
    }

    // SAFETY: the payload pointer is the one this module boxed when the callback was
    // registered.
    let signal_data = unsafe { &*(user_data as *const MenuItemObjectActivateData) };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Nautilus owns this pointer for the duration of the call, and
        // `from_raw_borrowed` rejects null and takes its own reference.
        if let Some(item) = unsafe { MenuItemObject::from_raw_borrowed(item) } {
            (signal_data.callback)(&item);
        }
    }));
}

pub(crate) unsafe extern "C" fn destroy_menu_item_object_activate_data(
    user_data: gpointer,
    _closure: *mut GClosure,
) {
    if !user_data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: the pointer is the one this module boxed for the callback and is
            // dropped exactly once.
            drop(unsafe { Box::from_raw(user_data as *mut MenuItemObjectActivateData) });
        }));
    }
}

pub(crate) struct ActivateData {
    pub(crate) activate_fn: Arc<dyn Fn(MenuActivation)>,
    pub(crate) target: MenuActivationTarget,
}

pub(crate) unsafe extern "C" fn menu_item_activate_trampoline(
    _item: *mut NautilusMenuItem,
    user_data: gpointer,
) {
    if user_data.is_null() {
        return;
    }

    // SAFETY: the payload pointer is the one this module boxed when the callback was
    // registered.
    let activate_data = unsafe { &*(user_data as *const ActivateData) };
    let activation = MenuActivation {
        target: activate_data.target.clone(),
    };

    let _ = catch_unwind(AssertUnwindSafe(|| {
        (activate_data.activate_fn)(activation);
    }));
}

pub(crate) unsafe extern "C" fn destroy_activate_data(
    user_data: gpointer,
    _closure: *mut GClosure,
) {
    if !user_data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: the pointer is the one this module boxed for the callback and is
            // dropped exactly once.
            drop(unsafe { Box::from_raw(user_data as *mut ActivateData) });
        }));
    }
}

pub(crate) fn connect_activate_signal(
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
    // SAFETY: the instance is the live object this wrapper owns, and the payload is paired
    // with the destroy notify that frees it.
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
        // SAFETY: the connect failed, so nothing took the payload and this scope still
        // owns the box it leaked a moment ago.
        unsafe {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                drop(Box::from_raw(activate_data));
            }));
        }
    }
}
