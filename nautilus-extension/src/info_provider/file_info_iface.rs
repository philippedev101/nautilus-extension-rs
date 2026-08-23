use super::*;

macro_rules! file_info_iface {
    (
        $iface_init_fn:ident,
        $is_gone_fn:ident,
        $get_name_fn:ident,
        $get_uri_fn:ident,
        $get_parent_uri_fn:ident,
        $get_uri_scheme_fn:ident,
        $get_mime_type_fn:ident,
        $is_mime_type_fn:ident,
        $is_directory_fn:ident,
        $add_emblem_fn:ident,
        $get_string_attribute_fn:ident,
        $add_string_attribute_fn:ident,
        $invalidate_extension_info_fn:ident,
        $get_activation_uri_fn:ident,
        $get_file_type_fn:ident,
        $get_location_fn:ident,
        $get_parent_location_fn:ident,
        $get_parent_info_fn:ident,
        $get_mount_fn:ident,
        $can_write_fn:ident,
        $rust_file_info:ident,
        $set_rust_file_info:ident,
        $clear_rust_file_info:ident
    ) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_file_info()` instead.
        pub(crate) unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusFileInfoInterface;
            // SAFETY: GObject calls this with a pointer to the vtable being initialised,
            // valid for the duration of the call.
            unsafe {
                (*iface_struct).is_gone = Some($is_gone_fn);
                (*iface_struct).get_name = Some($get_name_fn);
                (*iface_struct).get_uri = Some($get_uri_fn);
                (*iface_struct).get_parent_uri = Some($get_parent_uri_fn);
                (*iface_struct).get_uri_scheme = Some($get_uri_scheme_fn);
                (*iface_struct).get_mime_type = Some($get_mime_type_fn);
                (*iface_struct).is_mime_type = Some($is_mime_type_fn);
                (*iface_struct).is_directory = Some($is_directory_fn);
                (*iface_struct).add_emblem = Some($add_emblem_fn);
                (*iface_struct).get_string_attribute = Some($get_string_attribute_fn);
                (*iface_struct).add_string_attribute = Some($add_string_attribute_fn);
                (*iface_struct).invalidate_extension_info = Some($invalidate_extension_info_fn);
                (*iface_struct).get_activation_uri = Some($get_activation_uri_fn);
                (*iface_struct).get_file_type = Some($get_file_type_fn);
                (*iface_struct).get_location = Some($get_location_fn);
                (*iface_struct).get_parent_location = Some($get_parent_location_fn);
                (*iface_struct).get_parent_info = Some($get_parent_info_fn);
                (*iface_struct).get_mount = Some($get_mount_fn);
                (*iface_struct).can_write = Some($can_write_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $is_gone_fn(file_info: *mut NautilusFileInfo) -> gboolean {
            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.is_gone(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_name_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.name(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_uri_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.uri(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_parent_uri_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.parent_uri(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_uri_scheme_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.uri_scheme(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_mime_type_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.mime_type(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $is_mime_type_fn(
            file_info: *mut NautilusFileInfo,
            mime_type: *const c_char,
        ) -> gboolean {
            // SAFETY: Nautilus passes either null or a NUL-terminated C string, and
            // `c_string_arg` handles both.
            let mime_type = match unsafe { c_string_arg(mime_type) } {
                Some(mime_type) => mime_type,
                None => return GFALSE,
            };

            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.is_mime_type(handle, &mime_type),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $is_directory_fn(
            file_info: *mut NautilusFileInfo,
        ) -> gboolean {
            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.is_directory(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $add_emblem_fn(
            file_info: *mut NautilusFileInfo,
            emblem_name: *const c_char,
        ) {
            // SAFETY: Nautilus passes either null or a NUL-terminated C string, and
            // `c_string_arg` handles both.
            let emblem_name = match unsafe { c_string_arg(emblem_name) } {
                Some(emblem_name) => emblem_name,
                None => return,
            };

            with_file_info_impl(file_info, &$rust_file_info, (), |file_info, handle| {
                file_info.add_emblem(handle, &emblem_name);
            });
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_string_attribute_fn(
            file_info: *mut NautilusFileInfo,
            attribute_name: *const c_char,
        ) -> *mut c_char {
            // SAFETY: Nautilus passes either null or a NUL-terminated C string, and
            // `c_string_arg` handles both.
            let attribute_name = match unsafe { c_string_arg(attribute_name) } {
                Some(attribute_name) => attribute_name,
                None => return ptr::null_mut(),
            };

            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.string_attribute(handle, &attribute_name),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $add_string_attribute_fn(
            file_info: *mut NautilusFileInfo,
            attribute_name: *const c_char,
            value: *const c_char,
        ) {
            // SAFETY: Nautilus passes either null or a NUL-terminated C string, and
            // `c_string_arg` handles both.
            let attribute_name = match unsafe { c_string_arg(attribute_name) } {
                Some(attribute_name) => attribute_name,
                None => return,
            };
            // SAFETY: Nautilus passes either null or a NUL-terminated C string, and
            // `c_string_arg` handles both.
            let value = match unsafe { c_string_arg(value) } {
                Some(value) => value,
                None => return,
            };

            with_file_info_impl(file_info, &$rust_file_info, (), |file_info, handle| {
                file_info.add_string_attribute(handle, &attribute_name, &value);
            });
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $invalidate_extension_info_fn(
            file_info: *mut NautilusFileInfo,
        ) {
            with_file_info_impl(file_info, &$rust_file_info, (), |file_info, handle| {
                file_info.invalidate_extension_info(handle);
            });
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_activation_uri_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.activation_uri(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_file_type_fn(
            file_info: *mut NautilusFileInfo,
        ) -> GFileType {
            with_file_info_impl(
                file_info,
                &$rust_file_info,
                G_FILE_TYPE_UNKNOWN,
                |file_info, handle| file_info.file_type(handle),
            )
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_location_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut GFile {
            owned_gobject_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.location(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_parent_location_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut GFile {
            owned_gobject_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.parent_location(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_parent_info_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut NautilusFileInfo {
            file_info_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.parent_info(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $get_mount_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut GMount {
            owned_gobject_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.mount(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        pub(crate) unsafe extern "C" fn $can_write_fn(
            file_info: *mut NautilusFileInfo,
        ) -> gboolean {
            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.can_write(handle),
            ))
        }

        pub(crate) fn $set_rust_file_info(file_info: Box<dyn FileInfoImpl>) {
            if let Ok(mut current_file_info) = $rust_file_info.lock() {
                *current_file_info = Some(Arc::from(file_info));
            }
        }

        pub(crate) fn $clear_rust_file_info() {
            if let Ok(mut file_info) = $rust_file_info.lock() {
                *file_info = None;
            }
        }

        lazy_static! {
            pub(crate) static ref $rust_file_info: Mutex<Option<Arc<dyn FileInfoImpl>>> =
                Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_FILE_INFO_IMPLS: usize = 10;

#[rustfmt::skip] file_info_iface!(file_info_iface_init_0, file_info_is_gone_0, file_info_get_name_0, file_info_get_uri_0, file_info_get_parent_uri_0, file_info_get_uri_scheme_0, file_info_get_mime_type_0, file_info_is_mime_type_0, file_info_is_directory_0, file_info_add_emblem_0, file_info_get_string_attribute_0, file_info_add_string_attribute_0, file_info_invalidate_extension_info_0, file_info_get_activation_uri_0, file_info_get_file_type_0, file_info_get_location_0, file_info_get_parent_location_0, file_info_get_parent_info_0, file_info_get_mount_0, file_info_can_write_0, FILE_INFO_IMPL_0, set_file_info_impl_0, clear_file_info_impl_0);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_1, file_info_is_gone_1, file_info_get_name_1, file_info_get_uri_1, file_info_get_parent_uri_1, file_info_get_uri_scheme_1, file_info_get_mime_type_1, file_info_is_mime_type_1, file_info_is_directory_1, file_info_add_emblem_1, file_info_get_string_attribute_1, file_info_add_string_attribute_1, file_info_invalidate_extension_info_1, file_info_get_activation_uri_1, file_info_get_file_type_1, file_info_get_location_1, file_info_get_parent_location_1, file_info_get_parent_info_1, file_info_get_mount_1, file_info_can_write_1, FILE_INFO_IMPL_1, set_file_info_impl_1, clear_file_info_impl_1);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_2, file_info_is_gone_2, file_info_get_name_2, file_info_get_uri_2, file_info_get_parent_uri_2, file_info_get_uri_scheme_2, file_info_get_mime_type_2, file_info_is_mime_type_2, file_info_is_directory_2, file_info_add_emblem_2, file_info_get_string_attribute_2, file_info_add_string_attribute_2, file_info_invalidate_extension_info_2, file_info_get_activation_uri_2, file_info_get_file_type_2, file_info_get_location_2, file_info_get_parent_location_2, file_info_get_parent_info_2, file_info_get_mount_2, file_info_can_write_2, FILE_INFO_IMPL_2, set_file_info_impl_2, clear_file_info_impl_2);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_3, file_info_is_gone_3, file_info_get_name_3, file_info_get_uri_3, file_info_get_parent_uri_3, file_info_get_uri_scheme_3, file_info_get_mime_type_3, file_info_is_mime_type_3, file_info_is_directory_3, file_info_add_emblem_3, file_info_get_string_attribute_3, file_info_add_string_attribute_3, file_info_invalidate_extension_info_3, file_info_get_activation_uri_3, file_info_get_file_type_3, file_info_get_location_3, file_info_get_parent_location_3, file_info_get_parent_info_3, file_info_get_mount_3, file_info_can_write_3, FILE_INFO_IMPL_3, set_file_info_impl_3, clear_file_info_impl_3);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_4, file_info_is_gone_4, file_info_get_name_4, file_info_get_uri_4, file_info_get_parent_uri_4, file_info_get_uri_scheme_4, file_info_get_mime_type_4, file_info_is_mime_type_4, file_info_is_directory_4, file_info_add_emblem_4, file_info_get_string_attribute_4, file_info_add_string_attribute_4, file_info_invalidate_extension_info_4, file_info_get_activation_uri_4, file_info_get_file_type_4, file_info_get_location_4, file_info_get_parent_location_4, file_info_get_parent_info_4, file_info_get_mount_4, file_info_can_write_4, FILE_INFO_IMPL_4, set_file_info_impl_4, clear_file_info_impl_4);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_5, file_info_is_gone_5, file_info_get_name_5, file_info_get_uri_5, file_info_get_parent_uri_5, file_info_get_uri_scheme_5, file_info_get_mime_type_5, file_info_is_mime_type_5, file_info_is_directory_5, file_info_add_emblem_5, file_info_get_string_attribute_5, file_info_add_string_attribute_5, file_info_invalidate_extension_info_5, file_info_get_activation_uri_5, file_info_get_file_type_5, file_info_get_location_5, file_info_get_parent_location_5, file_info_get_parent_info_5, file_info_get_mount_5, file_info_can_write_5, FILE_INFO_IMPL_5, set_file_info_impl_5, clear_file_info_impl_5);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_6, file_info_is_gone_6, file_info_get_name_6, file_info_get_uri_6, file_info_get_parent_uri_6, file_info_get_uri_scheme_6, file_info_get_mime_type_6, file_info_is_mime_type_6, file_info_is_directory_6, file_info_add_emblem_6, file_info_get_string_attribute_6, file_info_add_string_attribute_6, file_info_invalidate_extension_info_6, file_info_get_activation_uri_6, file_info_get_file_type_6, file_info_get_location_6, file_info_get_parent_location_6, file_info_get_parent_info_6, file_info_get_mount_6, file_info_can_write_6, FILE_INFO_IMPL_6, set_file_info_impl_6, clear_file_info_impl_6);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_7, file_info_is_gone_7, file_info_get_name_7, file_info_get_uri_7, file_info_get_parent_uri_7, file_info_get_uri_scheme_7, file_info_get_mime_type_7, file_info_is_mime_type_7, file_info_is_directory_7, file_info_add_emblem_7, file_info_get_string_attribute_7, file_info_add_string_attribute_7, file_info_invalidate_extension_info_7, file_info_get_activation_uri_7, file_info_get_file_type_7, file_info_get_location_7, file_info_get_parent_location_7, file_info_get_parent_info_7, file_info_get_mount_7, file_info_can_write_7, FILE_INFO_IMPL_7, set_file_info_impl_7, clear_file_info_impl_7);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_8, file_info_is_gone_8, file_info_get_name_8, file_info_get_uri_8, file_info_get_parent_uri_8, file_info_get_uri_scheme_8, file_info_get_mime_type_8, file_info_is_mime_type_8, file_info_is_directory_8, file_info_add_emblem_8, file_info_get_string_attribute_8, file_info_add_string_attribute_8, file_info_invalidate_extension_info_8, file_info_get_activation_uri_8, file_info_get_file_type_8, file_info_get_location_8, file_info_get_parent_location_8, file_info_get_parent_info_8, file_info_get_mount_8, file_info_can_write_8, FILE_INFO_IMPL_8, set_file_info_impl_8, clear_file_info_impl_8);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_9, file_info_is_gone_9, file_info_get_name_9, file_info_get_uri_9, file_info_get_parent_uri_9, file_info_get_uri_scheme_9, file_info_get_mime_type_9, file_info_is_mime_type_9, file_info_is_directory_9, file_info_add_emblem_9, file_info_get_string_attribute_9, file_info_add_string_attribute_9, file_info_invalidate_extension_info_9, file_info_get_activation_uri_9, file_info_get_file_type_9, file_info_get_location_9, file_info_get_parent_location_9, file_info_get_parent_info_9, file_info_get_mount_9, file_info_can_write_9, FILE_INFO_IMPL_9, set_file_info_impl_9, clear_file_info_impl_9);

#[doc(hidden)]
pub fn file_info_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        file_info_iface_init_0,
        file_info_iface_init_1,
        file_info_iface_init_2,
        file_info_iface_init_3,
        file_info_iface_init_4,
        file_info_iface_init_5,
        file_info_iface_init_6,
        file_info_iface_init_7,
        file_info_iface_init_8,
        file_info_iface_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_file_info_impl_setters() -> Vec<fn(Box<dyn FileInfoImpl>)> {
    vec![
        set_file_info_impl_0,
        set_file_info_impl_1,
        set_file_info_impl_2,
        set_file_info_impl_3,
        set_file_info_impl_4,
        set_file_info_impl_5,
        set_file_info_impl_6,
        set_file_info_impl_7,
        set_file_info_impl_8,
        set_file_info_impl_9,
    ]
}

fn rust_file_info_impl_clearers() -> Vec<fn()> {
    vec![
        clear_file_info_impl_0,
        clear_file_info_impl_1,
        clear_file_info_impl_2,
        clear_file_info_impl_3,
        clear_file_info_impl_4,
        clear_file_info_impl_5,
        clear_file_info_impl_6,
        clear_file_info_impl_7,
        clear_file_info_impl_8,
        clear_file_info_impl_9,
    ]
}

static RESERVED_FILE_INFO_IFACE_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_file_info_iface_index() -> Option<usize> {
    take_next_slot(&RESERVED_FILE_INFO_IFACE_SLOTS, MAX_FILE_INFO_IMPLS)
}

pub(crate) fn release_file_info_iface_index(index: usize) {
    if let Some(clear_file_info) = rust_file_info_impl_clearers().get(index) {
        clear_file_info();
    }

    release_slot(&RESERVED_FILE_INFO_IFACE_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn file_info_impl_slot_is_set(index: usize) -> bool {
    match index {
        0 => FILE_INFO_IMPL_0
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        1 => FILE_INFO_IMPL_1
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        2 => FILE_INFO_IMPL_2
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        3 => FILE_INFO_IMPL_3
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        4 => FILE_INFO_IMPL_4
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        5 => FILE_INFO_IMPL_5
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        6 => FILE_INFO_IMPL_6
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        7 => FILE_INFO_IMPL_7
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        8 => FILE_INFO_IMPL_8
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        9 => FILE_INFO_IMPL_9
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_file_info_impl_state() {
    reset_slots(&RESERVED_FILE_INFO_IFACE_SLOTS);
    for clear_file_info in rust_file_info_impl_clearers() {
        clear_file_info();
    }
}
