use super::*;

macro_rules! menu_provider_iface {
    ($iface_init_fn:ident, $get_file_items_fn:ident, $get_background_items_fn:ident, $rust_provider:ident, $set_rust_provider:ident, $clear_rust_provider:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_menu_provider()` instead.
        pub(crate) unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusMenuProviderIface;
            unsafe {
                (*iface_struct).get_file_items = Some($get_file_items_fn);
                (*iface_struct).get_background_items = Some($get_background_items_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_file_items_fn(
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
        pub(crate) unsafe extern "C" fn $get_background_items_fn(
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

        pub(crate) fn $set_rust_provider(menu_provider: Box<dyn MenuProvider>) {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = Some(Arc::from(menu_provider));
            }
        }

        pub(crate) fn $clear_rust_provider() {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = None;
            }
        }

        lazy_static! {
            pub(crate) static ref $rust_provider: Mutex<Option<Arc<dyn MenuProvider>>> =
                Mutex::new(None);
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
