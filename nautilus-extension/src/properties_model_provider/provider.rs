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
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusPropertiesModelProvider` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_properties_model_provider_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusPropertiesModelProvider` GType.
    pub fn type_() -> GType {
        0
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

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Calls the provider interface and returns native model objects.
    pub fn get_models(&self, files: &[FileInfo]) -> Vec<PropertiesModelObject> {
        let mut raw_files: *mut GList = ptr::null_mut();
        for file in files {
            unsafe {
                raw_files = g_list_append(raw_files, file.raw() as *mut c_void);
            }
        }

        let models = unsafe { nautilus_properties_model_provider_get_models(self.raw, raw_files) };

        unsafe {
            g_list_free(raw_files);
        }

        let vec = unsafe {
            vec_from_g_list(models, |data| {
                PropertiesModelObject::from_raw_borrowed(data as *mut NautilusPropertiesModel)
            })
        };

        unsafe {
            free_owned_g_object_list(models);
        }

        vec
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Calls the provider interface and returns native model objects.
    pub fn get_models(&self, _files: &[FileInfo]) -> Vec<PropertiesModelObject> {
        Vec::new()
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
        unsafe {
            g_object_ref(self.raw as *mut RawGObject);
        }

        PropertiesModelProviderHandle { raw: self.raw }
    }
}

impl Drop for PropertiesModelProviderHandle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut RawGObject);
            }
        }
    }
}
