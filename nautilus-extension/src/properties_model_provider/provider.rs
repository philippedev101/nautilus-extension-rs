use super::*;

/// Provides model-backed properties sections for selected files.
///
/// Nautilus API 4 replaced GTK widget property pages with model-backed
/// properties sections. Return one or more [`PropertiesModel`] values, each
/// containing neutral [`PropertiesItem`] rows.
///
/// # Example
///
/// ```no_run
/// use nautilus_extension::{
///     FileInfo, PropertiesItem, PropertiesModel, PropertiesModelProvider,
/// };
///
/// struct Provider;
///
/// impl PropertiesModelProvider for Provider {
///     fn get_models(&self, files: &[FileInfo]) -> Vec<PropertiesModel> {
///         if files.len() != 1 {
///             return Vec::new();
///         }
///
///         let name_len = files[0].name().map(|name| name.chars().count()).unwrap_or(0);
///         vec![PropertiesModel::new(
///             "Example",
///             vec![PropertiesItem::new("Name length", name_len.to_string())],
///         )]
///     }
/// }
/// ```
pub trait PropertiesModelProvider: Send + Sync {
    /// Returns properties model sections for the selected files.
    fn get_models(&self, files: &[FileInfo]) -> Vec<PropertiesModel>;
}

#[derive(Debug)]
/// Owned handle for a `NautilusPropertiesModelProvider` instance.
pub struct PropertiesModelProviderHandle {
    raw: *mut NautilusPropertiesModelProvider,
}

impl PropertiesModelProviderHandle {
    /// Returns the registered `NautilusPropertiesModelProvider` GType.
    pub fn type_() -> GType {
        // SAFETY: the Nautilus GType registration functions take no arguments and are safe
        // to call at any point.
        unsafe { nautilus_properties_model_provider_get_type() }
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer
    /// `NautilusPropertiesModelProvider` GObject pointer. On success, the
    /// returned wrapper owns that reference.
    pub unsafe fn from_raw_full(
        raw: *mut NautilusPropertiesModelProvider,
    ) -> Option<PropertiesModelProviderHandle> {
        if raw.is_null() {
            return None;
        }

        Some(PropertiesModelProviderHandle { raw })
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusPropertiesModelProvider`
    /// GObject pointer. This function adds one reference and returns an owned
    /// wrapper for that reference.
    pub unsafe fn from_raw_borrowed(
        raw: *mut NautilusPropertiesModelProvider,
    ) -> Option<PropertiesModelProviderHandle> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: the caller guarantees the pointer is a live object, and null was rejected
        // just above. This takes the reference the returned wrapper owns.
        unsafe {
            g_object_ref(raw as *mut RawGObject);
        }
        Some(PropertiesModelProviderHandle { raw })
    }

    /// Returns the wrapped raw `NautilusPropertiesModelProvider` pointer.
    pub fn raw(&self) -> *mut NautilusPropertiesModelProvider {
        self.raw
    }

    /// Returns the wrapped raw `NautilusPropertiesModelProvider` pointer.
    pub fn as_ptr(&self) -> *mut NautilusPropertiesModelProvider {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusPropertiesModelProvider {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    /// Calls the provider interface and returns native model objects.
    pub fn get_models(&self, files: &[FileInfo]) -> Vec<PropertiesModelObject> {
        let mut raw_files: *mut GList = ptr::null_mut();
        for file in files {
            // SAFETY: the list is the one this function is building and the appended
            // pointer outlives the call.
            unsafe {
                raw_files = g_list_append(raw_files, file.raw() as *mut c_void);
            }
        }

        // SAFETY: `self.raw` is the live Nautilus object this wrapper owns.
        let models = unsafe { nautilus_properties_model_provider_get_models(self.raw, raw_files) };

        // SAFETY: the list was built here and holds only borrowed element pointers.
        unsafe {
            g_list_free(raw_files);
        }

        // SAFETY: `models` is the GList the provider interface just returned, whose
        // elements are `NautilusPropertiesModel` objects that stay alive until freed.
        let vec = unsafe {
            vec_from_g_list(models, |data| {
                PropertiesModelObject::from_raw_borrowed(data as *mut NautilusPropertiesModel)
            })
        };

        // SAFETY: the list came from the call just made and its elements are owned here.
        unsafe {
            free_owned_g_object_list(models);
        }

        vec
    }

    /// Calls the provider interface and returns native model objects.
    ///
    /// This is the Rust-style alias for
    /// [`PropertiesModelProviderHandle::get_models`].
    pub fn models(&self, files: &[FileInfo]) -> Vec<PropertiesModelObject> {
        self.get_models(files)
    }
}

impl Clone for PropertiesModelProviderHandle {
    fn clone(&self) -> PropertiesModelProviderHandle {
        // SAFETY: the wrapper holds a live reference to this object, so taking one more is
        // sound.
        unsafe {
            g_object_ref(self.raw as *mut RawGObject);
        }

        PropertiesModelProviderHandle { raw: self.raw }
    }
}

impl Drop for PropertiesModelProviderHandle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: the wrapper owns the reference being released and does not use the
            // pointer again.
            unsafe {
                g_object_unref(self.raw as *mut RawGObject);
            }
        }
    }
}
