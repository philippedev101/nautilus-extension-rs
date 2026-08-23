use crate::glib_ffi::{g_free, g_list_length, g_list_nth_data, gpointer, GList};
use crate::info_provider::FileInfo;
use crate::nautilus_ffi::NautilusFileInfo;
use std::ffi::CStr;
use std::os::raw::c_char;

pub fn file_info_vec_from_g_list(list: *mut GList) -> Vec<FileInfo> {
    let mut vec = Vec::new();

    if list.is_null() {
        return vec;
    }

    // SAFETY: `list` is either null or a GList owned by the caller, which these calls
    // tolerate.
    unsafe {
        let length = g_list_length(list);
        for i in 0..length {
            let raw_file_info = g_list_nth_data(list, i) as *mut NautilusFileInfo;
            if let Some(file_info) = FileInfo::from_raw_borrowed(raw_file_info) {
                vec.push(file_info);
            }
        }
    }

    vec
}

pub unsafe fn take_glib_string(raw: *mut c_char) -> Option<String> {
    if raw.is_null() {
        return None;
    }

    // SAFETY: `raw` is non-null, checked just above, and NUL-terminated by its producer.
    let value = unsafe { CStr::from_ptr(raw) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: `raw` is a GLib allocation this function owns.
    unsafe {
        g_free(raw as gpointer);
    }
    Some(value)
}

pub unsafe fn borrowed_string(raw: *const c_char) -> Option<String> {
    if raw.is_null() {
        return None;
    }

    Some(
        // SAFETY: `raw` is non-null, checked just above, and NUL-terminated by its
        // producer.
        unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned(),
    )
}

pub unsafe fn vec_from_g_list<T, F>(list: *mut GList, mut convert: F) -> Vec<T>
where
    F: FnMut(gpointer) -> Option<T>,
{
    let mut vec = Vec::new();

    if list.is_null() {
        return vec;
    }

    // SAFETY: `list` is either null or a GList owned by the caller, which these calls
    // tolerate.
    let length = unsafe { g_list_length(list) };
    for i in 0..length {
        // SAFETY: `list` is either null or a GList owned by the caller, which these calls
        // tolerate.
        if let Some(value) = convert(unsafe { g_list_nth_data(list, i) }) {
            vec.push(value);
        }
    }

    vec
}
