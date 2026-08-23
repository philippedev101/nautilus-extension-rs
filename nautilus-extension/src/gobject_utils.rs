use crate::glib_ffi::{g_list_free_full, gboolean, gpointer, GList, GQuark, GFALSE, GTRUE};
use crate::gobject_ffi::{g_object_get, g_object_set, g_object_unref, GObject};
use crate::translate::{borrowed_string, take_glib_string};
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

    /// Returns the live object and the property name as a C string, or `None`
    /// when either is unusable.
    fn property_target(&self, property: &str) -> Option<(*mut GObject, CString)> {
        let object = self.as_gobject();
        if object.is_null() {
            return None;
        }
        Some((object, CString::new(property).ok()?))
    }

    fn string_property(&self, property: &str) -> Option<String> {
        let (object, property) = self.property_target(property)?;
        let mut value: *mut c_char = ptr::null_mut();
        unsafe {
            g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
            take_glib_string(value)
        }
    }

    fn set_string_property(&self, property: &str, value: &str) -> bool {
        let Some((object, property)) = self.property_target(property) else {
            return false;
        };
        let Ok(value) = CString::new(value) else {
            return false;
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

    fn set_optional_string_property(&self, property: &str, value: Option<&str>) -> bool {
        let Some((object, property)) = self.property_target(property) else {
            return false;
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

    fn bool_property(&self, property: &str) -> bool {
        let Some((object, property)) = self.property_target(property) else {
            return false;
        };
        let mut value: gboolean = GFALSE;
        unsafe {
            g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
        }
        value != GFALSE
    }

    fn set_bool_property(&self, property: &str, value: bool) -> bool {
        let Some((object, property)) = self.property_target(property) else {
            return false;
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

    fn float_property(&self, property: &str) -> c_float {
        let Some((object, property)) = self.property_target(property) else {
            return 0.0;
        };
        let mut value: c_float = 0.0;
        unsafe {
            g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
        }
        value
    }

    fn set_double_property(&self, property: &str, value: c_double) -> bool {
        let Some((object, property)) = self.property_target(property) else {
            return false;
        };
        unsafe {
            g_object_set(object, property.as_ptr(), value, ptr::null::<c_char>());
        }
        true
    }

    fn int_property(&self, property: &str) -> c_int {
        let Some((object, property)) = self.property_target(property) else {
            return 0;
        };
        let mut value: c_int = 0;
        unsafe {
            g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
        }
        value
    }

    fn set_int_property(&self, property: &str, value: c_int) -> bool {
        let Some((object, property)) = self.property_target(property) else {
            return false;
        };
        unsafe {
            g_object_set(object, property.as_ptr(), value, ptr::null::<c_char>());
        }
        true
    }

    fn quark_property(&self, property: &str) -> GQuark {
        let Some((object, property)) = self.property_target(property) else {
            return 0;
        };
        let mut value: GQuark = 0;
        unsafe {
            g_object_get(object, property.as_ptr(), &mut value, ptr::null::<c_char>());
        }
        value
    }

    fn object_property<T>(&self, property: &str) -> Option<*mut T> {
        let (object, property) = self.property_target(property)?;
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
}

/// Safe access to a Nautilus object's C getters.
///
/// The object wrappers each hold a raw pointer and call getters through it, so
/// without this every accessor would open its own `unsafe` block to restate the
/// same invariant. Implementing this states it once; the methods below take the
/// C getter as an argument and are safe to call.
///
/// # Safety
///
/// [`NautilusObject::as_raw`] must return either null or a pointer to a live
/// object of type `Raw` that stays valid for the borrow of `self`. Null is
/// allowed because the wrappers are constructible from raw pointers; the
/// accessors below reject it rather than calling through it.
pub(crate) unsafe trait NautilusObject {
    /// The Nautilus C type this wrapper owns.
    type Raw;

    /// Returns the wrapped object as a raw pointer.
    fn as_raw(&self) -> *mut Self::Raw;

    /// Reads a getter that transfers ownership of a C string.
    ///
    /// The result is freed with `g_free`, so `get` must be transfer-full. In
    /// the sys crate that is the getters declared `-> *mut c_char`; the
    /// transfer-none ones return `*const c_char` and belong in
    /// [`NautilusObject::borrowed_string`].
    fn owned_string(
        &self,
        get: unsafe extern "C" fn(*mut Self::Raw) -> *mut c_char,
    ) -> Option<String> {
        let raw = self.as_raw();
        if raw.is_null() {
            return None;
        }
        unsafe { take_glib_string(get(raw)) }
    }

    /// Reads a getter that returns a borrowed C string.
    ///
    /// The result is copied and not freed, so `get` must be transfer-none: the
    /// sys crate declares those `-> *const c_char`.
    fn borrowed_string(
        &self,
        get: unsafe extern "C" fn(*mut Self::Raw) -> *const c_char,
    ) -> Option<String> {
        let raw = self.as_raw();
        if raw.is_null() {
            return None;
        }
        unsafe { borrowed_string(get(raw)) }
    }

    /// Reads a getter that returns a C boolean.
    fn flag(&self, get: unsafe extern "C" fn(*mut Self::Raw) -> gboolean) -> bool {
        let raw = self.as_raw();
        if raw.is_null() {
            return false;
        }
        unsafe { get(raw) != GFALSE }
    }

    /// Reads a getter that returns a plain C value.
    ///
    /// `fallback` is returned for a null object. It is passed in rather than
    /// taken from `Default` because zero is not the neutral value for every C
    /// enum: `NautilusOperationComplete` is 0, so a `Default` fallback would
    /// report success for a call that never happened.
    fn value_or<T>(&self, get: unsafe extern "C" fn(*mut Self::Raw) -> T, fallback: T) -> T {
        let raw = self.as_raw();
        if raw.is_null() {
            return fallback;
        }
        unsafe { get(raw) }
    }

    /// Reads a getter that returns a pointer, such as another GObject.
    fn pointer<T>(&self, get: unsafe extern "C" fn(*mut Self::Raw) -> *mut T) -> *mut T {
        let raw = self.as_raw();
        if raw.is_null() {
            return ptr::null_mut();
        }
        unsafe { get(raw) }
    }

    /// Calls a getter that takes one string argument and returns an owned string.
    ///
    /// As with [`NautilusObject::owned_string`], `get` must be transfer-full.
    fn owned_string_for(
        &self,
        get: unsafe extern "C" fn(*mut Self::Raw, *const c_char) -> *mut c_char,
        argument: &str,
    ) -> Option<String> {
        let raw = self.as_raw();
        if raw.is_null() {
            return None;
        }
        let argument = CString::new(argument).ok()?;
        unsafe { take_glib_string(get(raw, argument.as_ptr())) }
    }

    /// Calls a predicate that takes one string argument.
    fn flag_for(
        &self,
        get: unsafe extern "C" fn(*mut Self::Raw, *const c_char) -> gboolean,
        argument: &str,
    ) -> bool {
        let raw = self.as_raw();
        if raw.is_null() {
            return false;
        }
        let Ok(argument) = CString::new(argument) else {
            return false;
        };
        unsafe { get(raw, argument.as_ptr()) != GFALSE }
    }

    /// Calls a setter that takes one string argument.
    ///
    /// Returns whether the call was made: a null object or an argument that is
    /// not a valid C string leaves the object untouched.
    fn call_with_string(
        &self,
        call: unsafe extern "C" fn(*mut Self::Raw, *const c_char),
        argument: &str,
    ) -> bool {
        let raw = self.as_raw();
        if raw.is_null() {
            return false;
        }
        let Ok(argument) = CString::new(argument) else {
            return false;
        };
        unsafe { call(raw, argument.as_ptr()) }
        true
    }

    /// Calls a method that takes no arguments and returns nothing.
    fn call(&self, call: unsafe extern "C" fn(*mut Self::Raw)) {
        let raw = self.as_raw();
        if raw.is_null() {
            return;
        }
        unsafe { call(raw) }
    }
}
