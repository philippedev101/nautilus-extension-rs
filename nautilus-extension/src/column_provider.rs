use crate::glib_ffi::{g_list_append, gpointer, GList, GQuark, GType};
use crate::gobject_ffi::{g_object_ref, g_object_unref, GObject};
use crate::gobject_utils::{free_owned_g_object_list, GObjectProperties};
use crate::nautilus_ffi::{
    nautilus_column_get_type, nautilus_column_new, nautilus_column_provider_get_columns,
    nautilus_column_provider_get_type, NautilusColumn, NautilusColumnProvider,
    NautilusColumnProviderIface,
};
use crate::slot_allocator::{release_slot, reset_slots, take_next_slot};
use crate::translate::vec_from_g_list;
use libc::{c_int, c_void};
use std::borrow::Cow;
use std::ffi::CString;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

const COLUMN_ATTRIBUTE_Q_PROPERTY: &str = "attribute-q";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Sort direction values accepted by Nautilus column metadata.
pub enum ColumnSortOrder {
    /// Sort ascending by default.
    Ascending,
    /// Sort descending by default.
    Descending,
}

impl ColumnSortOrder {
    fn to_sort_type(self) -> c_int {
        match self {
            ColumnSortOrder::Ascending => 0,
            ColumnSortOrder::Descending => 1,
        }
    }

    fn from_sort_type(value: c_int) -> ColumnSortOrder {
        match value {
            1 => ColumnSortOrder::Descending,
            _ => ColumnSortOrder::Ascending,
        }
    }
}

#[derive(Clone, Debug)]
/// Descriptor for a custom list-view column.
///
/// The `attribute` must match the string attribute later set on each
/// [`crate::FileInfo`] by an [`crate::InfoProvider`].
pub struct Column {
    /// Globally unique column name, conventionally namespaced by the extension.
    pub name: Cow<'static, str>,
    /// File-info attribute displayed in this column.
    pub attribute: Cow<'static, str>,
    /// User-visible column label.
    pub label: Cow<'static, str>,
    /// User-visible column description.
    pub description: Cow<'static, str>,
    visible: Option<bool>,
    xalign: Option<f32>,
    default_sort_order: Option<ColumnSortOrder>,
}

impl Column {
    /// Creates a column descriptor.
    ///
    /// `attribute` must match the attribute name later set through
    /// [`crate::FileInfo::add_string_attribute`].
    pub fn new<N, A, L, D>(name: N, attribute: A, label: L, description: D) -> Column
    where
        N: Into<Cow<'static, str>>,
        A: Into<Cow<'static, str>>,
        L: Into<Cow<'static, str>>,
        D: Into<Cow<'static, str>>,
    {
        Column {
            name: name.into(),
            attribute: attribute.into(),
            label: label.into(),
            description: description.into(),
            visible: None,
            xalign: None,
            default_sort_order: None,
        }
    }

    /// Sets whether Nautilus should show the column by default.
    pub fn visible(mut self, visible: bool) -> Column {
        self.visible = Some(visible);
        self
    }

    /// Sets the horizontal alignment for column contents.
    pub fn xalign(mut self, xalign: f32) -> Column {
        self.xalign = Some(xalign);
        self
    }

    /// Sets the default sort order for this column.
    pub fn default_sort_order(mut self, sort_order: ColumnSortOrder) -> Column {
        self.default_sort_order = Some(sort_order);
        self
    }

    /// Returns the configured default visibility, if one was set.
    pub fn visible_value(&self) -> Option<bool> {
        self.visible
    }

    /// Returns the configured horizontal alignment, if one was set.
    pub fn xalign_value(&self) -> Option<f32> {
        self.xalign
    }

    /// Returns the configured default sort order, if one was set.
    pub fn default_sort_order_value(&self) -> Option<ColumnSortOrder> {
        self.default_sort_order
    }

    /// Builds the corresponding `NautilusColumn` object.
    pub fn to_object(&self) -> Option<ColumnObject> {
        let name = CString::new(&self.name as &str).ok()?;
        let attribute = CString::new(&self.attribute as &str).ok()?;
        let label = CString::new(&self.label as &str).ok()?;
        let description = CString::new(&self.description as &str).ok()?;

        let column = unsafe {
            nautilus_column_new(
                name.as_ptr(),
                attribute.as_ptr(),
                label.as_ptr(),
                description.as_ptr(),
            )
        };

        if column.is_null() {
            return None;
        }

        let column = unsafe { ColumnObject::from_raw_full(column) }?;

        if let Some(visible) = self.visible {
            column.set_visible(visible);
        }

        if let Some(xalign) = self.xalign {
            column.set_xalign(xalign);
        }

        if let Some(sort_order) = self.default_sort_order {
            column.set_default_sort_order(sort_order);
        }

        Some(column)
    }

    fn to_raw(&self) -> Option<*mut NautilusColumn> {
        self.to_object().map(ColumnObject::into_raw)
    }
}

#[derive(Debug)]
/// Owned reference to a `NautilusColumn` object.
pub struct ColumnObject {
    raw: *mut NautilusColumn,
}

impl ColumnObject {
    /// Returns the registered `NautilusColumn` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_column_get_type() }
    }

    /// Creates a native `NautilusColumn` object.
    pub fn new<N, A, L, D>(name: N, attribute: A, label: L, description: D) -> Option<ColumnObject>
    where
        N: AsRef<str>,
        A: AsRef<str>,
        L: AsRef<str>,
        D: AsRef<str>,
    {
        let name = CString::new(name.as_ref()).ok()?;
        let attribute = CString::new(attribute.as_ref()).ok()?;
        let label = CString::new(label.as_ref()).ok()?;
        let description = CString::new(description.as_ref()).ok()?;

        unsafe {
            ColumnObject::from_raw_full(nautilus_column_new(
                name.as_ptr(),
                attribute.as_ptr(),
                label.as_ptr(),
                description.as_ptr(),
            ))
        }
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusColumn` GObject pointer. This
    /// function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(raw: *mut NautilusColumn) -> Option<ColumnObject> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut GObject);
        }
        Some(ColumnObject { raw })
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer `NautilusColumn`
    /// GObject pointer. On success, the returned wrapper owns that reference.
    pub unsafe fn from_raw_full(raw: *mut NautilusColumn) -> Option<ColumnObject> {
        if raw.is_null() {
            return None;
        }

        Some(ColumnObject { raw })
    }

    /// Returns the wrapped raw `NautilusColumn` pointer.
    pub fn raw(&self) -> *mut NautilusColumn {
        self.raw
    }

    /// Returns the wrapped raw `NautilusColumn` pointer.
    pub fn as_ptr(&self) -> *mut NautilusColumn {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusColumn {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    /// Returns the column's unique name.
    pub fn name(&self) -> Option<String> {
        self.string_property("name")
    }

    /// Returns the file-info attribute displayed by the column.
    pub fn attribute(&self) -> Option<String> {
        self.string_property("attribute")
    }

    /// Returns the interned quark for the column attribute.
    pub fn attribute_q(&self) -> GQuark {
        self.quark_property(COLUMN_ATTRIBUTE_Q_PROPERTY)
    }

    /// Returns the user-visible column label.
    pub fn label(&self) -> Option<String> {
        self.string_property("label")
    }

    /// Returns the user-visible column description.
    pub fn description(&self) -> Option<String> {
        self.string_property("description")
    }

    /// Returns whether the column is visible by default.
    pub fn visible(&self) -> bool {
        self.bool_property("visible")
    }

    /// Returns the horizontal alignment for column contents.
    pub fn xalign(&self) -> f32 {
        self.float_property("xalign")
    }

    /// Returns the default sort order.
    pub fn default_sort_order(&self) -> ColumnSortOrder {
        ColumnSortOrder::from_sort_type(self.int_property("default-sort-order"))
    }

    /// Sets the file-info attribute displayed by the column.
    pub fn set_attribute(&self, attribute: &str) -> bool {
        self.set_string_property("attribute", attribute)
    }

    /// Sets the user-visible column label.
    pub fn set_label(&self, label: &str) -> bool {
        self.set_string_property("label", label)
    }

    /// Sets the user-visible column description.
    pub fn set_description(&self, description: &str) -> bool {
        self.set_string_property("description", description)
    }

    /// Sets whether the column is visible by default.
    pub fn set_visible(&self, visible: bool) -> bool {
        self.set_bool_property("visible", visible)
    }

    /// Sets the horizontal alignment for column contents.
    pub fn set_xalign(&self, xalign: f32) -> bool {
        self.set_double_property("xalign", xalign as f64)
    }

    /// Sets the default sort order.
    pub fn set_default_sort_order(&self, sort_order: ColumnSortOrder) -> bool {
        self.set_int_property("default-sort-order", sort_order.to_sort_type())
    }
}

// SAFETY: `raw` is null or a `NautilusColumn` this wrapper owns a reference to,
// so it stays a live GObject for as long as the wrapper is borrowed.
unsafe impl GObjectProperties for ColumnObject {
    fn as_gobject(&self) -> *mut GObject {
        self.raw as *mut GObject
    }
}

impl Clone for ColumnObject {
    fn clone(&self) -> ColumnObject {
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        ColumnObject { raw: self.raw }
    }
}

impl Drop for ColumnObject {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut GObject);
            }
        }
    }
}

/// Provides custom columns for Nautilus list views.
///
/// The method name mirrors Nautilus' `get_columns` interface method. The
/// returned [`Column`] objects describe columns only; use [`InfoProvider`] to
/// fill matching attributes on each file.
///
/// # Example
///
/// ```no_run
/// use nautilus_extension::{Column, ColumnProvider};
///
/// struct Provider;
///
/// impl ColumnProvider for Provider {
///     fn get_columns(&self) -> Vec<Column> {
///         vec![Column::new(
///             "Example::status",
///             "example_status",
///             "Example Status",
///             "Neutral example metadata",
///         )
///         .visible(true)]
///     }
/// }
/// ```
///
/// [`InfoProvider`]: crate::InfoProvider
pub trait ColumnProvider: Send + Sync {
    /// Returns the columns this extension contributes.
    fn get_columns(&self) -> Vec<Column>;
}

#[derive(Debug)]
/// Owned handle for a `NautilusColumnProvider` instance.
pub struct ColumnProviderHandle {
    raw: *mut NautilusColumnProvider,
}

impl ColumnProviderHandle {
    /// Returns the registered `NautilusColumnProvider` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_column_provider_get_type() }
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer
    /// `NautilusColumnProvider` GObject pointer. On success, the returned
    /// wrapper owns that reference.
    pub unsafe fn from_raw_full(raw: *mut NautilusColumnProvider) -> Option<ColumnProviderHandle> {
        if raw.is_null() {
            return None;
        }

        Some(ColumnProviderHandle { raw })
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusColumnProvider` GObject pointer.
    /// This function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(
        raw: *mut NautilusColumnProvider,
    ) -> Option<ColumnProviderHandle> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut GObject);
        }
        Some(ColumnProviderHandle { raw })
    }

    /// Returns the wrapped raw `NautilusColumnProvider` pointer.
    pub fn raw(&self) -> *mut NautilusColumnProvider {
        self.raw
    }

    /// Returns the wrapped raw `NautilusColumnProvider` pointer.
    pub fn as_ptr(&self) -> *mut NautilusColumnProvider {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusColumnProvider {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    /// Calls the provider interface and returns native column objects.
    pub fn get_columns(&self) -> Vec<ColumnObject> {
        let columns = unsafe { nautilus_column_provider_get_columns(self.raw) };
        let vec = unsafe {
            vec_from_g_list(columns, |data| {
                ColumnObject::from_raw_borrowed(data as *mut NautilusColumn)
            })
        };

        unsafe {
            free_owned_g_object_list(columns);
        }

        vec
    }

    /// Calls the provider interface and returns native column objects.
    ///
    /// This is the Rust-style alias for [`ColumnProviderHandle::get_columns`].
    pub fn columns(&self) -> Vec<ColumnObject> {
        self.get_columns()
    }
}

impl Clone for ColumnProviderHandle {
    fn clone(&self) -> ColumnProviderHandle {
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        ColumnProviderHandle { raw: self.raw }
    }
}

impl Drop for ColumnProviderHandle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut GObject);
            }
        }
    }
}

macro_rules! column_provider_iface {
    ($iface_init_fn:ident, $get_columns_fn:ident, $rust_provider:ident, $set_rust_provider:ident, $clear_rust_provider:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_column_provider()` instead.
        unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusColumnProviderIface;
            unsafe {
                (*iface_struct).get_columns = Some($get_columns_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_columns_fn(_provider: *mut NautilusColumnProvider) -> *mut GList {
            let rust_provider = $rust_provider
                .lock()
                .ok()
                .and_then(|provider| provider.clone());
            let columns = match rust_provider {
                Some(provider) => catch_unwind(AssertUnwindSafe(|| provider.get_columns()))
                    .unwrap_or_else(|_| Vec::new()),
                None => Vec::new(),
            };

            let mut columns_g_list = ptr::null_mut();

            for column in columns {
                if let Some(column) = column.to_raw() {
                    unsafe {
                        columns_g_list = g_list_append(columns_g_list, column as *mut c_void);
                    }
                }
            }

            columns_g_list
        }

        fn $set_rust_provider(column_provider: Box<dyn ColumnProvider>) {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = Some(Arc::from(column_provider));
            }
        }

        fn $clear_rust_provider() {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = None;
            }
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Arc<dyn ColumnProvider>>> = Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_COLUMN_PROVIDERS: usize = 10;

#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_0, column_provider_get_columns_0, COLUMN_PROVIDER_0, set_column_provider_0, clear_column_provider_0);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_1, column_provider_get_columns_1, COLUMN_PROVIDER_1, set_column_provider_1, clear_column_provider_1);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_2, column_provider_get_columns_2, COLUMN_PROVIDER_2, set_column_provider_2, clear_column_provider_2);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_3, column_provider_get_columns_3, COLUMN_PROVIDER_3, set_column_provider_3, clear_column_provider_3);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_4, column_provider_get_columns_4, COLUMN_PROVIDER_4, set_column_provider_4, clear_column_provider_4);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_5, column_provider_get_columns_5, COLUMN_PROVIDER_5, set_column_provider_5, clear_column_provider_5);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_6, column_provider_get_columns_6, COLUMN_PROVIDER_6, set_column_provider_6, clear_column_provider_6);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_7, column_provider_get_columns_7, COLUMN_PROVIDER_7, set_column_provider_7, clear_column_provider_7);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_8, column_provider_get_columns_8, COLUMN_PROVIDER_8, set_column_provider_8, clear_column_provider_8);
#[rustfmt::skip] column_provider_iface!(column_provider_iface_init_9, column_provider_get_columns_9, COLUMN_PROVIDER_9, set_column_provider_9, clear_column_provider_9);

#[doc(hidden)]
pub fn column_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        column_provider_iface_init_0,
        column_provider_iface_init_1,
        column_provider_iface_init_2,
        column_provider_iface_init_3,
        column_provider_iface_init_4,
        column_provider_iface_init_5,
        column_provider_iface_init_6,
        column_provider_iface_init_7,
        column_provider_iface_init_8,
        column_provider_iface_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_column_provider_setters() -> Vec<fn(Box<dyn ColumnProvider>)> {
    vec![
        set_column_provider_0,
        set_column_provider_1,
        set_column_provider_2,
        set_column_provider_3,
        set_column_provider_4,
        set_column_provider_5,
        set_column_provider_6,
        set_column_provider_7,
        set_column_provider_8,
        set_column_provider_9,
    ]
}

fn rust_column_provider_clearers() -> Vec<fn()> {
    vec![
        clear_column_provider_0,
        clear_column_provider_1,
        clear_column_provider_2,
        clear_column_provider_3,
        clear_column_provider_4,
        clear_column_provider_5,
        clear_column_provider_6,
        clear_column_provider_7,
        clear_column_provider_8,
        clear_column_provider_9,
    ]
}

static RESERVED_COLUMN_PROVIDER_IFACE_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_column_provider_iface_index() -> Option<usize> {
    take_next_slot(&RESERVED_COLUMN_PROVIDER_IFACE_SLOTS, MAX_COLUMN_PROVIDERS)
}

pub(crate) fn release_column_provider_iface_index(index: usize) {
    if let Some(clear_provider) = rust_column_provider_clearers().get(index) {
        clear_provider();
    }

    release_slot(&RESERVED_COLUMN_PROVIDER_IFACE_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn column_provider_slot_is_set(index: usize) -> bool {
    match index {
        0 => COLUMN_PROVIDER_0
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        1 => COLUMN_PROVIDER_1
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        2 => COLUMN_PROVIDER_2
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        3 => COLUMN_PROVIDER_3
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        4 => COLUMN_PROVIDER_4
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        5 => COLUMN_PROVIDER_5
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        6 => COLUMN_PROVIDER_6
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        7 => COLUMN_PROVIDER_7
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        8 => COLUMN_PROVIDER_8
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        9 => COLUMN_PROVIDER_9
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_column_provider_state() {
    reset_slots(&RESERVED_COLUMN_PROVIDER_IFACE_SLOTS);
    for clear_provider in rust_column_provider_clearers() {
        clear_provider();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{require_native_api, require_unlinked_build};
    use std::sync::atomic::Ordering;

    static GET_COLUMNS_CALLS: AtomicUsize = AtomicUsize::new(0);

    struct RoutedColumnProvider;

    impl ColumnProvider for RoutedColumnProvider {
        fn get_columns(&self) -> Vec<Column> {
            GET_COLUMNS_CALLS.fetch_add(1, Ordering::SeqCst);
            Vec::new()
        }
    }

    struct PanickingColumnProvider;

    impl ColumnProvider for PanickingColumnProvider {
        fn get_columns(&self) -> Vec<Column> {
            panic!("column provider callback panic");
        }
    }

    #[test]
    fn column_builder_keeps_all_metadata() {
        let column = Column::new(
            "Example::status",
            "example_status",
            "Status",
            "Example status",
        )
        .visible(false)
        .xalign(0.75)
        .default_sort_order(ColumnSortOrder::Descending);

        assert_eq!(column.name.as_ref(), "Example::status");
        assert_eq!(column.attribute.as_ref(), "example_status");
        assert_eq!(column.label.as_ref(), "Status");
        assert_eq!(column.description.as_ref(), "Example status");
        assert_eq!(column.visible_value(), Some(false));
        assert_eq!(column.xalign_value(), Some(0.75));
        assert_eq!(
            column.default_sort_order_value(),
            Some(ColumnSortOrder::Descending)
        );
    }

    #[test]
    fn sort_order_maps_to_gtk_sort_type_values() {
        assert_eq!(ColumnSortOrder::Ascending.to_sort_type(), 0);
        assert_eq!(ColumnSortOrder::Descending.to_sort_type(), 1);
        assert_eq!(
            ColumnSortOrder::from_sort_type(0),
            ColumnSortOrder::Ascending
        );
        assert_eq!(
            ColumnSortOrder::from_sort_type(1),
            ColumnSortOrder::Descending
        );
        assert_eq!(
            ColumnSortOrder::from_sort_type(99),
            ColumnSortOrder::Ascending
        );
    }

    #[test]
    fn column_object_rejects_null_raw_pointers() {
        assert!(unsafe { ColumnObject::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { ColumnObject::from_raw_borrowed(ptr::null_mut()) }.is_none());
        assert!(unsafe { ColumnProviderHandle::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { ColumnProviderHandle::from_raw_borrowed(ptr::null_mut()) }.is_none());
    }

    #[test]
    fn attribute_q_uses_documented_property_name() {
        assert_eq!(COLUMN_ATTRIBUTE_Q_PROPERTY, "attribute-q");
    }

    #[test]
    fn column_provider_iface_trampoline_routes_to_registered_impl() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_column_provider_state();
        GET_COLUMNS_CALLS.store(0, Ordering::SeqCst);

        set_column_provider_0(Box::new(RoutedColumnProvider));

        let mut iface: NautilusColumnProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            column_provider_iface_init_0(
                &mut iface as *mut NautilusColumnProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let columns = unsafe { (iface.get_columns.unwrap())(ptr::null_mut()) };

        assert!(columns.is_null());
        assert_eq!(GET_COLUMNS_CALLS.load(Ordering::SeqCst), 1);

        reset_column_provider_state();
    }

    #[test]
    fn column_provider_iface_trampoline_catches_provider_panics() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_column_provider_state();

        set_column_provider_0(Box::new(PanickingColumnProvider));

        let mut iface: NautilusColumnProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            column_provider_iface_init_0(
                &mut iface as *mut NautilusColumnProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let result =
            std::panic::catch_unwind(|| unsafe { (iface.get_columns.unwrap())(ptr::null_mut()) });

        assert_eq!(result.unwrap(), ptr::null_mut());

        reset_column_provider_state();
    }

    #[test]
    fn documented_type_accessors_are_inert_in_no_link_mode() {
        require_unlinked_build!();

        assert_eq!(ColumnObject::type_(), 0);
        assert_eq!(ColumnProviderHandle::type_(), 0);
    }

    #[test]
    fn documented_type_accessors_return_registered_gtypes() {
        require_native_api!();

        assert_ne!(ColumnObject::type_(), 0);
        assert_ne!(ColumnProviderHandle::type_(), 0);
    }

    #[test]
    fn reset_column_provider_state_allows_slot_reuse() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_column_provider_state();

        for expected_index in 0..MAX_COLUMN_PROVIDERS {
            assert_eq!(
                take_next_column_provider_iface_index(),
                Some(expected_index)
            );
        }

        assert_eq!(take_next_column_provider_iface_index(), None);

        reset_column_provider_state();

        assert_eq!(take_next_column_provider_iface_index(), Some(0));

        reset_column_provider_state();
    }

    #[test]
    fn native_object_construction_is_inert_in_no_link_mode() {
        require_unlinked_build!();

        assert!(ColumnObject::new("Example::status", "example_status", "Status", "Desc").is_none());
        assert!(
            Column::new("Example::status", "example_status", "Status", "Desc")
                .to_object()
                .is_none()
        );
    }

    #[test]
    fn native_object_construction_round_trips_with_the_native_library() {
        require_native_api!();

        let column = Column::new("Example::status", "example_status", "Status", "Desc")
            .to_object()
            .expect("column should be constructible");

        assert_eq!(column.label().as_deref(), Some("Status"));
    }

    #[test]
    fn accessors_on_a_null_object_never_reach_glib() {
        // Reaching GLib with a null object logs a critical, and the test run
        // makes criticals fatal, so this aborts if a guard is ever dropped.
        let column = ColumnObject {
            raw: ptr::null_mut(),
        };

        assert_eq!(column.name(), None);
        assert_eq!(column.attribute(), None);
        assert_eq!(column.label(), None);
        assert_eq!(column.description(), None);
        assert_eq!(column.attribute_q(), 0);
        assert!(!column.visible());
        assert_eq!(column.xalign(), 0.0);
        assert_eq!(column.default_sort_order(), ColumnSortOrder::Ascending);

        assert!(!column.set_attribute("example_status"));
        assert!(!column.set_label("Status"));
        assert!(!column.set_description("Example status"));
        assert!(!column.set_visible(true));
        assert!(!column.set_xalign(0.5));
        assert!(!column.set_default_sort_order(ColumnSortOrder::Descending));
    }
}
