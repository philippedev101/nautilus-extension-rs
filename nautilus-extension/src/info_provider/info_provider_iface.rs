use super::*;

macro_rules! info_provider_iface {
    ($iface_init_fn:ident, $update_file_info_fn:ident, $cancel_update_fn:ident, $rust_provider:ident, $set_rust_provider:ident, $clear_rust_provider:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_info_provider()` instead.
        pub(crate) unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusInfoProviderIface;
            unsafe {
                (*iface_struct).update_file_info = Some($update_file_info_fn);
                (*iface_struct).cancel_update = Some($cancel_update_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $update_file_info_fn(
            provider: *mut NautilusInfoProvider,
            file: *mut NautilusFileInfo,
            update_complete: *mut GClosure,
            handle: *mut *mut NautilusOperationHandle,
        ) -> NautilusOperationResult {
            let rust_provider = $rust_provider
                .lock()
                .ok()
                .and_then(|provider| provider.clone());
            let rust_provider = match rust_provider {
                Some(provider) => provider,
                None => return OperationResult::Complete.into(),
            };

            let (operation, raw_handle, state) =
                match new_operation(provider, file, update_complete) {
                    Some(operation) => operation,
                    None => return OperationResult::Failed.into(),
                };

            if !register_in_flight(raw_handle, state.clone()) {
                cleanup_state(&state);
                drop_raw_operation_handle(raw_handle);
                return OperationResult::Failed.into();
            }

            let result = catch_unwind(AssertUnwindSafe(|| {
                rust_provider.update_file_info_full(operation)
            }))
            .unwrap_or(OperationResult::Failed);

            if result == OperationResult::InProgress && !state.cancelled.load(Ordering::SeqCst) {
                if handle.is_null() {
                    finish_in_flight(raw_handle);
                    cleanup_state(&state);
                    return OperationResult::Failed.into();
                }

                unsafe {
                    *handle = raw_handle;
                }
                state.accepted.store(true, Ordering::SeqCst);
                OperationResult::InProgress.into()
            } else {
                finish_in_flight(raw_handle);
                cleanup_state(&state);
                result.into()
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $cancel_update_fn(
            _provider: *mut NautilusInfoProvider,
            handle: *mut NautilusOperationHandle,
        ) {
            let rust_provider = $rust_provider
                .lock()
                .ok()
                .and_then(|provider| provider.clone());
            let operation_handle = lookup_handle(handle);

            if let Some(state) = operation_handle.state.as_ref() {
                state.cancelled.store(true, Ordering::SeqCst);
            }

            if let Some(provider) = rust_provider {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    provider.cancel_update(&operation_handle);
                }));
            }

            if let Some(state) = operation_handle.state.as_ref() {
                finish_in_flight(handle);
                cleanup_state(state);
            }
        }

        pub(crate) fn $set_rust_provider(info_provider: Box<dyn InfoProvider>) {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = Some(Arc::from(info_provider));
            }
        }

        pub(crate) fn $clear_rust_provider() {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = None;
            }
        }

        lazy_static! {
            pub(crate) static ref $rust_provider: Mutex<Option<Arc<dyn InfoProvider>>> =
                Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_INFO_PROVIDERS: usize = 10;

#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_0, info_provider_update_file_info_0, info_provider_cancel_update_0, INFO_PROVIDER_0, set_info_provider_0, clear_info_provider_0);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_1, info_provider_update_file_info_1, info_provider_cancel_update_1, INFO_PROVIDER_1, set_info_provider_1, clear_info_provider_1);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_2, info_provider_update_file_info_2, info_provider_cancel_update_2, INFO_PROVIDER_2, set_info_provider_2, clear_info_provider_2);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_3, info_provider_update_file_info_3, info_provider_cancel_update_3, INFO_PROVIDER_3, set_info_provider_3, clear_info_provider_3);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_4, info_provider_update_file_info_4, info_provider_cancel_update_4, INFO_PROVIDER_4, set_info_provider_4, clear_info_provider_4);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_5, info_provider_update_file_info_5, info_provider_cancel_update_5, INFO_PROVIDER_5, set_info_provider_5, clear_info_provider_5);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_6, info_provider_update_file_info_6, info_provider_cancel_update_6, INFO_PROVIDER_6, set_info_provider_6, clear_info_provider_6);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_7, info_provider_update_file_info_7, info_provider_cancel_update_7, INFO_PROVIDER_7, set_info_provider_7, clear_info_provider_7);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_8, info_provider_update_file_info_8, info_provider_cancel_update_8, INFO_PROVIDER_8, set_info_provider_8, clear_info_provider_8);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_9, info_provider_update_file_info_9, info_provider_cancel_update_9, INFO_PROVIDER_9, set_info_provider_9, clear_info_provider_9);

#[doc(hidden)]
pub fn info_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        info_provider_iface_init_0,
        info_provider_iface_init_1,
        info_provider_iface_init_2,
        info_provider_iface_init_3,
        info_provider_iface_init_4,
        info_provider_iface_init_5,
        info_provider_iface_init_6,
        info_provider_iface_init_7,
        info_provider_iface_init_8,
        info_provider_iface_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_info_provider_setters() -> Vec<fn(Box<dyn InfoProvider>)> {
    vec![
        set_info_provider_0,
        set_info_provider_1,
        set_info_provider_2,
        set_info_provider_3,
        set_info_provider_4,
        set_info_provider_5,
        set_info_provider_6,
        set_info_provider_7,
        set_info_provider_8,
        set_info_provider_9,
    ]
}

fn rust_info_provider_clearers() -> Vec<fn()> {
    vec![
        clear_info_provider_0,
        clear_info_provider_1,
        clear_info_provider_2,
        clear_info_provider_3,
        clear_info_provider_4,
        clear_info_provider_5,
        clear_info_provider_6,
        clear_info_provider_7,
        clear_info_provider_8,
        clear_info_provider_9,
    ]
}

static RESERVED_INFO_PROVIDER_IFACE_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_info_provider_iface_index() -> Option<usize> {
    take_next_slot(&RESERVED_INFO_PROVIDER_IFACE_SLOTS, MAX_INFO_PROVIDERS)
}

pub(crate) fn release_info_provider_iface_index(index: usize) {
    if let Some(clear_provider) = rust_info_provider_clearers().get(index) {
        clear_provider();
    }

    release_slot(&RESERVED_INFO_PROVIDER_IFACE_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn info_provider_slot_is_set(index: usize) -> bool {
    match index {
        0 => INFO_PROVIDER_0
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        1 => INFO_PROVIDER_1
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        2 => INFO_PROVIDER_2
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        3 => INFO_PROVIDER_3
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        4 => INFO_PROVIDER_4
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        5 => INFO_PROVIDER_5
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        6 => INFO_PROVIDER_6
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        7 => INFO_PROVIDER_7
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        8 => INFO_PROVIDER_8
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        9 => INFO_PROVIDER_9
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_info_provider_state() {
    reset_slots(&RESERVED_INFO_PROVIDER_IFACE_SLOTS);

    for clear_provider in rust_info_provider_clearers() {
        clear_provider();
    }

    let operations: Vec<(usize, Arc<OperationState>)> = match IN_FLIGHT_OPERATIONS.lock() {
        Ok(mut operations) => operations.drain().collect(),
        Err(_) => Vec::new(),
    };

    for (raw_handle, state) in operations {
        state.cancelled.store(true, Ordering::SeqCst);
        cleanup_state(&state);
        drop_raw_operation_handle(raw_handle as *mut NautilusOperationHandle);
    }
}
