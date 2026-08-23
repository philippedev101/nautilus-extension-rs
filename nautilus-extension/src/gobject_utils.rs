use crate::glib_ffi::{g_list_free_full, gboolean, gpointer, GList, GQuark, GFALSE, GTRUE};
use crate::gobject_ffi::{g_object_get, g_object_set, g_object_unref, GObject};
use crate::translate::take_glib_string;
use libc::{c_char, c_double, c_float, c_int};
use std::ffi::CString;
use std::ptr;

pub unsafe extern "C" fn unref_g_object(data: gpointer) {
    if !data.is_null() {
        unsafe {
            g_object_unref(data as *mut GObject);
        }
    }
}

pub unsafe fn free_owned_g_object_list(list: *mut GList) {
    if !list.is_null() {
        unsafe {
            g_list_free_full(list, Some(unref_g_object));
        }
    }
}

pub unsafe fn get_string_property(object: *mut GObject, property: &str) -> Option<String> {
    let property = CString::new(property).ok()?;
    let mut value: *mut c_char = ptr::null_mut();

    unsafe {
        g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
    }

    unsafe { take_glib_string(value) }
}

pub unsafe fn set_string_property(object: *mut GObject, property: &str, value: &str) -> bool {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return false,
    };
    let value = match CString::new(value) {
        Ok(value) => value,
        Err(_) => return false,
    };

    unsafe {
        g_object_set(
            object,
            property.as_ptr(),
            value.as_ptr(),
            ptr::null::<c_char>(),
        );
    }
    true
}

pub unsafe fn set_optional_string_property(
    object: *mut GObject,
    property: &str,
    value: Option<&str>,
) -> bool {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return false,
    };
    let value = match value {
        Some(value) => match CString::new(value) {
            Ok(value) => Some(value),
            Err(_) => return false,
        },
        None => None,
    };

    unsafe {
        g_object_set(
            object,
            property.as_ptr(),
            value
                .as_ref()
                .map(|value| value.as_ptr())
                .unwrap_or(ptr::null()),
            ptr::null::<c_char>(),
        );
    }

    true
}

pub unsafe fn get_bool_property(object: *mut GObject, property: &str) -> bool {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return false,
    };
    let mut value: gboolean = GFALSE;

    unsafe {
        g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
    }

    value != GFALSE
}

pub unsafe fn set_bool_property(object: *mut GObject, property: &str, value: bool) -> bool {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return false,
    };

    unsafe {
        g_object_set(
            object,
            property.as_ptr(),
            if value { GTRUE } else { GFALSE },
            ptr::null::<c_char>(),
        );
    }

    true
}

pub unsafe fn get_float_property(object: *mut GObject, property: &str) -> c_float {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return 0.0,
    };
    let mut value: c_float = 0.0;

    unsafe {
        g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
    }

    value
}

pub unsafe fn set_double_property(object: *mut GObject, property: &str, value: c_double) -> bool {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return false,
    };

    unsafe {
        g_object_set(object, property.as_ptr(), value, ptr::null::<c_char>());
    }
    true
}

pub unsafe fn get_int_property(object: *mut GObject, property: &str) -> c_int {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return 0,
    };
    let mut value: c_int = 0;

    unsafe {
        g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
    }

    value
}

pub unsafe fn set_int_property(object: *mut GObject, property: &str, value: c_int) -> bool {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return false,
    };

    unsafe {
        g_object_set(object, property.as_ptr(), value, ptr::null::<c_char>());
    }
    true
}

pub unsafe fn get_quark_property(object: *mut GObject, property: &str) -> GQuark {
    let property = match CString::new(property) {
        Ok(property) => property,
        Err(_) => return 0,
    };
    let mut value: GQuark = 0;

    unsafe {
        g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
    }

    value
}

pub unsafe fn get_object_property<T>(object: *mut GObject, property: &str) -> Option<*mut T> {
    let property = CString::new(property).ok()?;
    let mut value: *mut T = ptr::null_mut();

    unsafe {
        g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
    }

    if value.is_null() {
        None
    } else {
        Some(value)
    }
}
