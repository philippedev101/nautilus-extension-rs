use super::*;

macro_rules! properties_model_provider_iface {
    ($iface_init_fn:ident, $get_models_fn:ident, $rust_provider:ident, $set_rust_provider:ident, $clear_rust_provider:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_properties_model_provider()` instead.
        pub(crate) unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusPropertiesModelProviderIface;
            unsafe {
                (*iface_struct).get_models = Some($get_models_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_models_fn(
            _provider: *mut NautilusPropertiesModelProvider,
            raw_files: *mut GList,
        ) -> *mut GList {
            let rust_provider = $rust_provider
                .lock()
                .ok()
                .and_then(|provider| provider.clone());
            let files = file_info_vec_from_g_list(raw_files);
            let models = match rust_provider {
                Some(provider) => catch_unwind(AssertUnwindSafe(|| provider.get_models(&files)))
                    .unwrap_or_else(|_| Vec::new()),
                None => Vec::new(),
            };

            let mut models_g_list = ptr::null_mut();

            for model in models {
                if let Some(raw_model) = model.to_raw() {
                    unsafe {
                        models_g_list = g_list_append(models_g_list, raw_model as *mut c_void);
                    }
                }
            }

            models_g_list
        }

        pub(crate) fn $set_rust_provider(provider: Box<dyn PropertiesModelProvider>) {
            if let Ok(mut rust_provider) = $rust_provider.lock() {
                *rust_provider = Some(Arc::from(provider));
            }
        }

        pub(crate) fn $clear_rust_provider() {
            if let Ok(mut rust_provider) = $rust_provider.lock() {
                *rust_provider = None;
            }
        }

        lazy_static! {
            pub(crate) static ref $rust_provider: Mutex<Option<Arc<dyn PropertiesModelProvider>>> =
                Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_PROPERTIES_MODEL_PROVIDERS: usize = 10;

#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_0, properties_model_provider_get_models_0, PROPERTIES_MODEL_PROVIDER_0, set_properties_model_provider_0, clear_properties_model_provider_0);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_1, properties_model_provider_get_models_1, PROPERTIES_MODEL_PROVIDER_1, set_properties_model_provider_1, clear_properties_model_provider_1);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_2, properties_model_provider_get_models_2, PROPERTIES_MODEL_PROVIDER_2, set_properties_model_provider_2, clear_properties_model_provider_2);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_3, properties_model_provider_get_models_3, PROPERTIES_MODEL_PROVIDER_3, set_properties_model_provider_3, clear_properties_model_provider_3);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_4, properties_model_provider_get_models_4, PROPERTIES_MODEL_PROVIDER_4, set_properties_model_provider_4, clear_properties_model_provider_4);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_5, properties_model_provider_get_models_5, PROPERTIES_MODEL_PROVIDER_5, set_properties_model_provider_5, clear_properties_model_provider_5);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_6, properties_model_provider_get_models_6, PROPERTIES_MODEL_PROVIDER_6, set_properties_model_provider_6, clear_properties_model_provider_6);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_7, properties_model_provider_get_models_7, PROPERTIES_MODEL_PROVIDER_7, set_properties_model_provider_7, clear_properties_model_provider_7);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_8, properties_model_provider_get_models_8, PROPERTIES_MODEL_PROVIDER_8, set_properties_model_provider_8, clear_properties_model_provider_8);
#[rustfmt::skip] properties_model_provider_iface!(properties_model_provider_iface_init_9, properties_model_provider_get_models_9, PROPERTIES_MODEL_PROVIDER_9, set_properties_model_provider_9, clear_properties_model_provider_9);

#[doc(hidden)]
pub fn properties_model_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        properties_model_provider_iface_init_0,
        properties_model_provider_iface_init_1,
        properties_model_provider_iface_init_2,
        properties_model_provider_iface_init_3,
        properties_model_provider_iface_init_4,
        properties_model_provider_iface_init_5,
        properties_model_provider_iface_init_6,
        properties_model_provider_iface_init_7,
        properties_model_provider_iface_init_8,
        properties_model_provider_iface_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_properties_model_provider_setters() -> Vec<fn(Box<dyn PropertiesModelProvider>)> {
    vec![
        set_properties_model_provider_0,
        set_properties_model_provider_1,
        set_properties_model_provider_2,
        set_properties_model_provider_3,
        set_properties_model_provider_4,
        set_properties_model_provider_5,
        set_properties_model_provider_6,
        set_properties_model_provider_7,
        set_properties_model_provider_8,
        set_properties_model_provider_9,
    ]
}

fn rust_properties_model_provider_clearers() -> Vec<fn()> {
    vec![
        clear_properties_model_provider_0,
        clear_properties_model_provider_1,
        clear_properties_model_provider_2,
        clear_properties_model_provider_3,
        clear_properties_model_provider_4,
        clear_properties_model_provider_5,
        clear_properties_model_provider_6,
        clear_properties_model_provider_7,
        clear_properties_model_provider_8,
        clear_properties_model_provider_9,
    ]
}

static RESERVED_PROPERTIES_MODEL_PROVIDER_IFACE_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_properties_model_provider_iface_index() -> Option<usize> {
    take_next_slot(
        &RESERVED_PROPERTIES_MODEL_PROVIDER_IFACE_SLOTS,
        MAX_PROPERTIES_MODEL_PROVIDERS,
    )
}

pub(crate) fn release_properties_model_provider_iface_index(index: usize) {
    if let Some(clear_provider) = rust_properties_model_provider_clearers().get(index) {
        clear_provider();
    }

    release_slot(&RESERVED_PROPERTIES_MODEL_PROVIDER_IFACE_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn properties_model_provider_slot_is_set(index: usize) -> bool {
    match index {
        0 => PROPERTIES_MODEL_PROVIDER_0
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        1 => PROPERTIES_MODEL_PROVIDER_1
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        2 => PROPERTIES_MODEL_PROVIDER_2
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        3 => PROPERTIES_MODEL_PROVIDER_3
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        4 => PROPERTIES_MODEL_PROVIDER_4
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        5 => PROPERTIES_MODEL_PROVIDER_5
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        6 => PROPERTIES_MODEL_PROVIDER_6
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        7 => PROPERTIES_MODEL_PROVIDER_7
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        8 => PROPERTIES_MODEL_PROVIDER_8
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        9 => PROPERTIES_MODEL_PROVIDER_9
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_properties_model_provider_state() {
    reset_slots(&RESERVED_PROPERTIES_MODEL_PROVIDER_IFACE_SLOTS);
    for clear_provider in rust_properties_model_provider_clearers() {
        clear_provider();
    }
}
