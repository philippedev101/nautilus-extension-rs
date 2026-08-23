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

/// Safe GObject property access for the owned object wrappers.
///
/// The wrappers all hold a raw GObject pointer and all read and write
/// properties the same way, which previously meant an `unsafe` block at every
/// accessor. Implementing this states the invariant once instead, and every
/// method below is then safe to call.
///
/// # Safety
///
/// [`GObjectProperties::as_gobject`] must return either null or a pointer to a
/// live GObject that stays valid for the borrow of `self`. Null is allowed
/// because the wrappers are constructible from raw pointers; the accessors
/// below reject it rather than handing it to GLib.
pub(crate) unsafe trait GObjectProperties {
    /// Returns the wrapped object as a raw GObject pointer.
    fn as_gobject(&self) -> *mut GObject;

    fn string_property(&self, property: &str) -> Option<String> {
        let object = self.as_gobject();
        if object.is_null() {
            return None;
        }
        unsafe { get_string_property(object, property) }
    }

    fn set_string_property(&self, property: &str, value: &str) -> bool {
        let object = self.as_gobject();
        if object.is_null() {
            return false;
        }
        unsafe { set_string_property(object, property, value) }
    }

    fn set_optional_string_property(&self, property: &str, value: Option<&str>) -> bool {
        let object = self.as_gobject();
        if object.is_null() {
            return false;
        }
        unsafe { set_optional_string_property(object, property, value) }
    }

    fn bool_property(&self, property: &str) -> bool {
        let object = self.as_gobject();
        if object.is_null() {
            return false;
        }
        unsafe { get_bool_property(object, property) }
    }

    fn set_bool_property(&self, property: &str, value: bool) -> bool {
        let object = self.as_gobject();
        if object.is_null() {
            return false;
        }
        unsafe { set_bool_property(object, property, value) }
    }

    fn float_property(&self, property: &str) -> c_float {
        let object = self.as_gobject();
        if object.is_null() {
            return 0.0;
        }
        unsafe { get_float_property(object, property) }
    }

    fn set_double_property(&self, property: &str, value: c_double) -> bool {
        let object = self.as_gobject();
        if object.is_null() {
            return false;
        }
        unsafe { set_double_property(object, property, value) }
    }

    fn int_property(&self, property: &str) -> c_int {
        let object = self.as_gobject();
        if object.is_null() {
            return 0;
        }
        unsafe { get_int_property(object, property) }
    }

    fn set_int_property(&self, property: &str, value: c_int) -> bool {
        let object = self.as_gobject();
        if object.is_null() {
            return false;
        }
        unsafe { set_int_property(object, property, value) }
    }

    fn object_property<T>(&self, property: &str) -> Option<*mut T> {
        let object = self.as_gobject();
        if object.is_null() {
            return None;
        }
        unsafe { get_object_property(object, property) }
    }

    fn quark_property(&self, property: &str) -> GQuark {
        let object = self.as_gobject();
        if object.is_null() {
            return 0;
        }
        unsafe { get_quark_property(object, property) }
    }
}
