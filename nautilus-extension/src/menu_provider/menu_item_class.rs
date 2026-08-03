use super::*;

macro_rules! menu_item_class {
    ($class_init_fn:ident, $activate_fn:ident, $rust_activate:ident, $set_rust_activate:ident, $clear_rust_activate:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a GObject class initializer. Do not call directly.
        /// Use `NautilusModule.try_register_menu_item_type()` instead.
        pub(crate) unsafe extern "C" fn $class_init_fn(class: gpointer, _: gpointer) {
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
        pub(crate) unsafe extern "C" fn $activate_fn(item: *mut NautilusMenuItem) {
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

        pub(crate) fn $set_rust_activate(activate: Box<dyn MenuItemActivate>) {
            if let Ok(mut current_activate) = $rust_activate.lock() {
                *current_activate = Some(Arc::from(activate));
            }
        }

        pub(crate) fn $clear_rust_activate() {
            if let Ok(mut activate) = $rust_activate.lock() {
                *activate = None;
            }
        }

        lazy_static! {
            pub(crate) static ref $rust_activate: Mutex<Option<Arc<dyn MenuItemActivate>>> =
                Mutex::new(None);
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
