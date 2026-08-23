use super::*;

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

    /// Builds the corresponding native `NautilusPropertiesItem` object.
    pub fn to_object(&self) -> Option<PropertiesItemObject> {
        let name = CString::new(&self.name as &str).ok()?;
        let value = CString::new(&self.value as &str).ok()?;

        let item = unsafe { nautilus_properties_item_new(name.as_ptr(), value.as_ptr()) };

        unsafe { PropertiesItemObject::from_raw_full(item) }
    }

    fn to_raw(&self) -> Option<*mut NautilusPropertiesItem> {
        self.to_object().map(PropertiesItemObject::into_raw)
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

    /// Builds the corresponding native `NautilusPropertiesModel` object.
    pub fn to_object(&self) -> Option<PropertiesModelObject> {
        if !NATIVE_API_AVAILABLE {
            return None;
        }

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

    pub(crate) fn to_raw(&self) -> Option<*mut NautilusPropertiesModel> {
        self.to_object().map(PropertiesModelObject::into_raw)
    }
}

#[derive(Debug)]
/// Owned reference to a `NautilusPropertiesItem` object.
pub struct PropertiesItemObject {
    raw: *mut NautilusPropertiesItem,
}

impl PropertiesItemObject {
    /// Returns the registered `NautilusPropertiesItem` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_properties_item_get_type() }
    }

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

    /// Returns the row name.
    pub fn name(&self) -> Option<String> {
        unsafe { borrowed_string(nautilus_properties_item_get_name(self.raw)) }
    }

    /// Returns the row value.
    pub fn value(&self) -> Option<String> {
        unsafe { borrowed_string(nautilus_properties_item_get_value(self.raw)) }
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
    /// Returns the registered `NautilusPropertiesModel` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_properties_model_get_type() }
    }

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

    /// Returns the user-visible section title.
    pub fn title(&self) -> Option<String> {
        unsafe { borrowed_string(nautilus_properties_model_get_title(self.raw)) }
    }

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

    /// Returns the underlying `GListModel`.
    pub fn model(&self) -> Option<OwnedGObject<GListModel>> {
        unsafe { OwnedGObject::from_raw_borrowed(nautilus_properties_model_get_model(self.raw)) }
    }

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
