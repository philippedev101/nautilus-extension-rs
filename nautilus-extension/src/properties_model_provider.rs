use crate::gio_ffi::GListModel;
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gio_ffi::{
    g_list_model_get_item, g_list_model_get_n_items, g_list_store_append, g_list_store_new,
};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::glib_ffi::g_list_free;
use crate::glib_ffi::{g_list_append, gpointer, GList, GType};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gobject_ffi::GObject;
use crate::gobject_ffi::{g_object_ref, g_object_unref, GObject as RawGObject};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::gobject_utils::free_owned_g_object_list;
use crate::info_provider::{FileInfo, OwnedGObject};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::nautilus_ffi::{
    nautilus_properties_item_get_name, nautilus_properties_item_get_type,
    nautilus_properties_item_get_value, nautilus_properties_item_new,
    nautilus_properties_model_get_model, nautilus_properties_model_get_title,
    nautilus_properties_model_get_type, nautilus_properties_model_new,
    nautilus_properties_model_provider_get_models, nautilus_properties_model_provider_get_type,
    nautilus_properties_model_set_title,
};
use crate::nautilus_ffi::{
    NautilusPropertiesItem, NautilusPropertiesModel, NautilusPropertiesModelProvider,
    NautilusPropertiesModelProviderIface,
};
use crate::slot_allocator::{release_slot, reset_slots, take_next_slot};
use crate::translate::file_info_vec_from_g_list;
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::translate::{borrowed_string, vec_from_g_list};
use libc::c_void;
use std::borrow::Cow;
#[cfg(not(nautilus_extension_rs_skip_link))]
use std::ffi::CString;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
/// One name/value row in a Nautilus properties model.
pub struct PropertiesItem {
    /// User-visible row name.
    pub name: Cow<'static, str>,
    /// User-visible row value.
    pub value: Cow<'static, str>,
}

impl PropertiesItem {
    /// Creates a properties row.
    pub fn new<N, V>(name: N, value: V) -> PropertiesItem
    where
        N: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        PropertiesItem {
            name: name.into(),
            value: value.into(),
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Builds the corresponding native `NautilusPropertiesItem` object.
    pub fn to_object(&self) -> Option<PropertiesItemObject> {
        let name = CString::new(&self.name as &str).ok()?;
        let value = CString::new(&self.value as &str).ok()?;

        let item = unsafe { nautilus_properties_item_new(name.as_ptr(), value.as_ptr()) };

        unsafe { PropertiesItemObject::from_raw_full(item) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[allow(dead_code)]
    /// Builds the corresponding native `NautilusPropertiesItem` object.
    pub fn to_object(&self) -> Option<PropertiesItemObject> {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    fn to_raw(&self) -> Option<*mut NautilusPropertiesItem> {
        self.to_object().map(PropertiesItemObject::into_raw)
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[allow(dead_code)]
    fn to_raw(&self) -> Option<*mut NautilusPropertiesItem> {
        None
    }
}

#[derive(Clone, Debug)]
/// A model-backed properties section shown in Nautilus' Properties dialog.
pub struct PropertiesModel {
    /// User-visible section title.
    pub title: Cow<'static, str>,
    /// Rows shown in the section.
    pub items: Vec<PropertiesItem>,
}

impl PropertiesModel {
    /// Creates a properties model section.
    pub fn new<T: Into<Cow<'static, str>>>(
        title: T,
        items: Vec<PropertiesItem>,
    ) -> PropertiesModel {
        PropertiesModel {
            title: title.into(),
            items,
        }
    }

    /// Appends a row to this model section.
    pub fn push_item(&mut self, item: PropertiesItem) -> &mut PropertiesModel {
        self.items.push(item);
        self
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Builds the corresponding native `NautilusPropertiesModel` object.
    pub fn to_object(&self) -> Option<PropertiesModelObject> {
        let title = CString::new(&self.title as &str).ok()?;
        let store = unsafe { g_list_store_new(nautilus_properties_item_get_type()) };

        if store.is_null() {
            return None;
        }

        for item in &self.items {
            if let Some(raw_item) = item.to_raw() {
                unsafe {
                    g_list_store_append(store, raw_item as *mut GObject);
                    g_object_unref(raw_item as *mut GObject);
                }
            }
        }

        let model =
            unsafe { nautilus_properties_model_new(title.as_ptr(), store as *mut GListModel) };

        unsafe {
            g_object_unref(store as *mut GObject);
        }

        unsafe { PropertiesModelObject::from_raw_full(model) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Builds the corresponding native `NautilusPropertiesModel` object.
    pub fn to_object(&self) -> Option<PropertiesModelObject> {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    fn to_raw(&self) -> Option<*mut NautilusPropertiesModel> {
        self.to_object().map(PropertiesModelObject::into_raw)
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    fn to_raw(&self) -> Option<*mut NautilusPropertiesModel> {
        None
    }
}

#[derive(Debug)]
/// Owned reference to a `NautilusPropertiesItem` object.
pub struct PropertiesItemObject {
    raw: *mut NautilusPropertiesItem,
}

impl PropertiesItemObject {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusPropertiesItem` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_properties_item_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusPropertiesItem` GType.
    pub fn type_() -> GType {
        0
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Creates a native properties item object.
    pub fn new<N, V>(name: N, value: V) -> Option<PropertiesItemObject>
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        let name = CString::new(name.as_ref()).ok()?;
        let value = CString::new(value.as_ref()).ok()?;

        unsafe {
            PropertiesItemObject::from_raw_full(nautilus_properties_item_new(
                name.as_ptr(),
                value.as_ptr(),
            ))
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Creates a native properties item object.
    pub fn new<N, V>(_name: N, _value: V) -> Option<PropertiesItemObject>
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        None
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusPropertiesItem` GObject pointer.
    /// This function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(
        raw: *mut NautilusPropertiesItem,
    ) -> Option<PropertiesItemObject> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut RawGObject);
        }
        Some(PropertiesItemObject { raw })
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer
    /// `NautilusPropertiesItem` GObject pointer. On success, the returned
    /// wrapper owns that reference.
    pub unsafe fn from_raw_full(raw: *mut NautilusPropertiesItem) -> Option<PropertiesItemObject> {
        if raw.is_null() {
            return None;
        }

        Some(PropertiesItemObject { raw })
    }

    /// Returns the wrapped raw `NautilusPropertiesItem` pointer.
    pub fn raw(&self) -> *mut NautilusPropertiesItem {
        self.raw
    }

    /// Returns the wrapped raw `NautilusPropertiesItem` pointer.
    pub fn as_ptr(&self) -> *mut NautilusPropertiesItem {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusPropertiesItem {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the row name.
    pub fn name(&self) -> Option<String> {
        unsafe { borrowed_string(nautilus_properties_item_get_name(self.raw)) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the row name.
    pub fn name(&self) -> Option<String> {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the row value.
    pub fn value(&self) -> Option<String> {
        unsafe { borrowed_string(nautilus_properties_item_get_value(self.raw)) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the row value.
    pub fn value(&self) -> Option<String> {
        None
    }
}

impl Clone for PropertiesItemObject {
    fn clone(&self) -> PropertiesItemObject {
        unsafe {
            g_object_ref(self.raw as *mut RawGObject);
        }

        PropertiesItemObject { raw: self.raw }
    }
}

impl Drop for PropertiesItemObject {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut RawGObject);
            }
        }
    }
}

#[derive(Debug)]
/// Owned reference to a `NautilusPropertiesModel` object.
pub struct PropertiesModelObject {
    raw: *mut NautilusPropertiesModel,
}

impl PropertiesModelObject {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusPropertiesModel` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_properties_model_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusPropertiesModel` GType.
    pub fn type_() -> GType {
        0
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Creates a native properties model object from an existing list model.
    pub fn new<T: AsRef<str>>(
        title: T,
        model: &OwnedGObject<GListModel>,
    ) -> Option<PropertiesModelObject> {
        let title = CString::new(title.as_ref()).ok()?;

        unsafe {
            PropertiesModelObject::from_raw_full(nautilus_properties_model_new(
                title.as_ptr(),
                model.as_ptr(),
            ))
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Creates a native properties model object from an existing list model.
    pub fn new<T: AsRef<str>>(
        _title: T,
        _model: &OwnedGObject<GListModel>,
    ) -> Option<PropertiesModelObject> {
        None
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusPropertiesModel` GObject
    /// pointer. This function adds one reference and returns an owned wrapper
    /// for that reference.
    pub unsafe fn from_raw_borrowed(
        raw: *mut NautilusPropertiesModel,
    ) -> Option<PropertiesModelObject> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut RawGObject);
        }
        Some(PropertiesModelObject { raw })
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer
    /// `NautilusPropertiesModel` GObject pointer. On success, the returned
    /// wrapper owns that reference.
    pub unsafe fn from_raw_full(
        raw: *mut NautilusPropertiesModel,
    ) -> Option<PropertiesModelObject> {
        if raw.is_null() {
            return None;
        }

        Some(PropertiesModelObject { raw })
    }

    /// Returns the wrapped raw `NautilusPropertiesModel` pointer.
    pub fn raw(&self) -> *mut NautilusPropertiesModel {
        self.raw
    }

    /// Returns the wrapped raw `NautilusPropertiesModel` pointer.
    pub fn as_ptr(&self) -> *mut NautilusPropertiesModel {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusPropertiesModel {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the user-visible section title.
    pub fn title(&self) -> Option<String> {
        unsafe { borrowed_string(nautilus_properties_model_get_title(self.raw)) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the user-visible section title.
    pub fn title(&self) -> Option<String> {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Sets the user-visible section title.
    pub fn set_title(&self, title: &str) -> bool {
        let title = match CString::new(title) {
            Ok(title) => title,
            Err(_) => return false,
        };

        unsafe {
            nautilus_properties_model_set_title(self.raw, title.as_ptr());
        }

        true
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Sets the user-visible section title.
    pub fn set_title(&self, _title: &str) -> bool {
        false
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the underlying `GListModel`.
    pub fn model(&self) -> Option<OwnedGObject<GListModel>> {
        unsafe { OwnedGObject::from_raw_borrowed(nautilus_properties_model_get_model(self.raw)) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the underlying `GListModel`.
    pub fn model(&self) -> Option<OwnedGObject<GListModel>> {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the properties items in the underlying model.
    pub fn items(&self) -> Vec<PropertiesItemObject> {
        let model = match self.model() {
            Some(model) => model,
            None => return Vec::new(),
        };

        let mut items = Vec::new();

        unsafe {
            let len = g_list_model_get_n_items(model.as_ptr());
            for index in 0..len {
                let item = g_list_model_get_item(model.as_ptr(), index);
                if let Some(item) =
                    PropertiesItemObject::from_raw_full(item as *mut NautilusPropertiesItem)
                {
                    items.push(item);
                }
            }
        }

        items
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the properties items in the underlying model.
    pub fn items(&self) -> Vec<PropertiesItemObject> {
        Vec::new()
    }
}

impl Clone for PropertiesModelObject {
    fn clone(&self) -> PropertiesModelObject {
        unsafe {
            g_object_ref(self.raw as *mut RawGObject);
        }

        PropertiesModelObject { raw: self.raw }
    }
}

impl Drop for PropertiesModelObject {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut RawGObject);
            }
        }
    }
}

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

macro_rules! properties_model_provider_iface {
    ($iface_init_fn:ident, $get_models_fn:ident, $rust_provider:ident, $set_rust_provider:ident, $clear_rust_provider:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_properties_model_provider()` instead.
        unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusPropertiesModelProviderIface;
            unsafe {
                (*iface_struct).get_models = Some($get_models_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_models_fn(
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

        fn $set_rust_provider(provider: Box<dyn PropertiesModelProvider>) {
            if let Ok(mut rust_provider) = $rust_provider.lock() {
                *rust_provider = Some(Arc::from(provider));
            }
        }

        fn $clear_rust_provider() {
            if let Ok(mut rust_provider) = $rust_provider.lock() {
                *rust_provider = None;
            }
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Arc<dyn PropertiesModelProvider>>> =
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(nautilus_extension_rs_skip_link)]
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(nautilus_extension_rs_skip_link)]
    static GET_MODELS_CALLS: AtomicUsize = AtomicUsize::new(0);

    #[cfg(nautilus_extension_rs_skip_link)]
    struct RoutedPropertiesModelProvider;

    #[cfg(nautilus_extension_rs_skip_link)]
    impl PropertiesModelProvider for RoutedPropertiesModelProvider {
        fn get_models(&self, files: &[FileInfo]) -> Vec<PropertiesModel> {
            assert!(files.is_empty());
            GET_MODELS_CALLS.fetch_add(1, Ordering::SeqCst);
            vec![PropertiesModel::new(
                "Example",
                vec![PropertiesItem::new("Name", "Value")],
            )]
        }
    }

    struct PanickingPropertiesModelProvider;

    impl PropertiesModelProvider for PanickingPropertiesModelProvider {
        fn get_models(&self, _files: &[FileInfo]) -> Vec<PropertiesModel> {
            panic!("properties model provider callback panic");
        }
    }

    #[test]
    fn properties_item_keeps_name_and_value() {
        let item = PropertiesItem::new("URI scheme", "file");

        assert_eq!(item.name.as_ref(), "URI scheme");
        assert_eq!(item.value.as_ref(), "file");
    }

    #[test]
    fn properties_model_can_append_items() {
        let mut model =
            PropertiesModel::new("Example", vec![PropertiesItem::new("Name length", "12")]);

        model.push_item(PropertiesItem::new("MIME type", "text/plain"));

        assert_eq!(model.title.as_ref(), "Example");
        assert_eq!(model.items.len(), 2);
        assert_eq!(model.items[0].name.as_ref(), "Name length");
        assert_eq!(model.items[1].value.as_ref(), "text/plain");
    }

    #[test]
    fn properties_object_wrappers_reject_null_raw_pointers() {
        assert!(unsafe { PropertiesItemObject::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { PropertiesItemObject::from_raw_borrowed(ptr::null_mut()) }.is_none());
        assert!(unsafe { PropertiesModelObject::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { PropertiesModelObject::from_raw_borrowed(ptr::null_mut()) }.is_none());
        assert!(unsafe { PropertiesModelProviderHandle::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(
            unsafe { PropertiesModelProviderHandle::from_raw_borrowed(ptr::null_mut()) }.is_none()
        );
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn properties_model_provider_iface_trampoline_routes_to_registered_impl() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_properties_model_provider_state();
        GET_MODELS_CALLS.store(0, Ordering::SeqCst);

        set_properties_model_provider_0(Box::new(RoutedPropertiesModelProvider));

        let mut iface: NautilusPropertiesModelProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            properties_model_provider_iface_init_0(
                &mut iface as *mut NautilusPropertiesModelProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let models = unsafe { (iface.get_models.unwrap())(ptr::null_mut(), ptr::null_mut()) };

        assert!(models.is_null());
        assert_eq!(GET_MODELS_CALLS.load(Ordering::SeqCst), 1);

        reset_properties_model_provider_state();
    }

    #[test]
    fn properties_model_provider_iface_trampoline_catches_provider_panics() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_properties_model_provider_state();

        set_properties_model_provider_0(Box::new(PanickingPropertiesModelProvider));

        let mut iface: NautilusPropertiesModelProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            properties_model_provider_iface_init_0(
                &mut iface as *mut NautilusPropertiesModelProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let result = std::panic::catch_unwind(|| unsafe {
            (iface.get_models.unwrap())(ptr::null_mut(), ptr::null_mut())
        });

        assert_eq!(result.unwrap(), ptr::null_mut());

        reset_properties_model_provider_state();
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn documented_type_accessors_are_inert_in_no_link_mode() {
        assert_eq!(PropertiesItemObject::type_(), 0);
        assert_eq!(PropertiesModelObject::type_(), 0);
        assert_eq!(PropertiesModelProviderHandle::type_(), 0);
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn documented_type_accessors_return_registered_gtypes() {
        assert_ne!(PropertiesItemObject::type_(), 0);
        assert_ne!(PropertiesModelObject::type_(), 0);
        assert_ne!(PropertiesModelProviderHandle::type_(), 0);
    }

    #[test]
    fn reset_properties_model_provider_state_allows_slot_reuse() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_properties_model_provider_state();

        for expected_index in 0..MAX_PROPERTIES_MODEL_PROVIDERS {
            assert_eq!(
                take_next_properties_model_provider_iface_index(),
                Some(expected_index)
            );
        }

        assert_eq!(take_next_properties_model_provider_iface_index(), None);

        reset_properties_model_provider_state();

        assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

        reset_properties_model_provider_state();
    }
}
