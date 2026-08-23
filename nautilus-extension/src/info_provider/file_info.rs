use super::*;

#[derive(Debug)]
/// Owned reference to a `NautilusFileInfo`.
pub struct FileInfo {
    pub(crate) raw_file_info: *mut NautilusFileInfo,
}

impl FileInfo {
    /// Returns the registered `NautilusFileInfo` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_file_info_get_type() }
    }

    /// # Safety
    ///
    /// `raw_file_info` must be a valid borrowed `NautilusFileInfo` GObject
    /// pointer. This function adds one reference and returns an owned wrapper
    /// for that reference.
    pub unsafe fn from_raw_borrowed(raw_file_info: *mut NautilusFileInfo) -> Option<FileInfo> {
        if raw_file_info.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw_file_info as *mut GObject);
        }
        Some(FileInfo { raw_file_info })
    }

    /// # Safety
    ///
    /// `raw_file_info` must be either null or a valid full-transfer
    /// `NautilusFileInfo` GObject pointer. On success, the returned wrapper
    /// owns that reference.
    pub unsafe fn from_raw_full(raw_file_info: *mut NautilusFileInfo) -> Option<FileInfo> {
        if raw_file_info.is_null() {
            return None;
        }

        Some(FileInfo { raw_file_info })
    }

    /// Creates a `FileInfo` for a Gio file location.
    pub fn create(location: &OwnedGObject<GFile>) -> Option<FileInfo> {
        unsafe { FileInfo::from_raw_full(nautilus_file_info_create(location.as_ptr())) }
    }

    /// Creates a `FileInfo` for a URI.
    pub fn create_for_uri(uri: &str) -> Option<FileInfo> {
        let uri = CString::new(uri).ok()?;
        unsafe { FileInfo::from_raw_full(nautilus_file_info_create_for_uri(uri.as_ptr())) }
    }

    /// Looks up an existing `FileInfo` for a Gio file location.
    pub fn lookup(location: &OwnedGObject<GFile>) -> Option<FileInfo> {
        unsafe { FileInfo::from_raw_full(nautilus_file_info_lookup(location.as_ptr())) }
    }

    /// Looks up an existing `FileInfo` for a URI.
    pub fn lookup_for_uri(uri: &str) -> Option<FileInfo> {
        let uri = CString::new(uri).ok()?;
        unsafe { FileInfo::from_raw_full(nautilus_file_info_lookup_for_uri(uri.as_ptr())) }
    }

    /// Returns the wrapped raw `NautilusFileInfo` pointer.
    pub fn raw(&self) -> *mut NautilusFileInfo {
        self.raw_file_info
    }

    /// Returns the wrapped raw `NautilusFileInfo` pointer.
    pub fn as_ptr(&self) -> *mut NautilusFileInfo {
        self.raw_file_info
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusFileInfo {
        let raw = self.raw_file_info;
        self.raw_file_info = ptr::null_mut();
        raw
    }

    /// Returns whether Nautilus considers the file gone.
    pub fn is_gone(&self) -> bool {
        self.flag(nautilus_file_info_is_gone)
    }

    /// Returns the display name for the file.
    pub fn name(&self) -> Option<String> {
        self.owned_string(nautilus_file_info_get_name)
    }

    /// Returns the file URI.
    pub fn uri(&self) -> Option<String> {
        self.owned_string(nautilus_file_info_get_uri)
    }

    /// Returns the parent directory URI.
    pub fn parent_uri(&self) -> Option<String> {
        self.owned_string(nautilus_file_info_get_parent_uri)
    }

    /// Returns the URI scheme, such as `file` or `trash`.
    pub fn uri_scheme(&self) -> Option<String> {
        self.owned_string(nautilus_file_info_get_uri_scheme)
    }

    /// Returns the MIME type known by Nautilus.
    pub fn mime_type(&self) -> Option<String> {
        self.owned_string(nautilus_file_info_get_mime_type)
    }

    /// Returns whether the file matches `mime_type`.
    pub fn is_mime_type(&self, mime_type: &str) -> bool {
        self.flag_for(nautilus_file_info_is_mime_type, mime_type)
    }

    /// Returns whether the file is a directory.
    pub fn is_directory(&self) -> bool {
        self.flag(nautilus_file_info_is_directory)
    }

    /// Adds an emblem by icon name.
    pub fn add_emblem(&self, emblem_name: &str) {
        self.call_with_string(nautilus_file_info_add_emblem, emblem_name)
    }

    /// Returns a string attribute previously known to Nautilus.
    pub fn string_attribute(&self, attribute_name: &str) -> Option<String> {
        self.owned_string_for(nautilus_file_info_get_string_attribute, attribute_name)
    }

    /// Adds or updates a string attribute for this file.
    pub fn add_string_attribute(&self, attribute_name: &str, value: &str) {
        let attribute_name = match CString::new(attribute_name) {
            Ok(attribute_name) => attribute_name,
            Err(_) => return,
        };
        let value = match CString::new(value) {
            Ok(value) => value,
            Err(_) => return,
        };

        unsafe {
            nautilus_file_info_add_string_attribute(
                self.raw_file_info,
                attribute_name.as_ptr(),
                value.as_ptr(),
            );
        }
    }

    /// Alias for [`FileInfo::add_string_attribute`].
    pub fn add_attribute(&self, attribute_name: &str, value: &str) {
        self.add_string_attribute(attribute_name, value);
    }

    /// Asks Nautilus to refresh extension-provided info for this file.
    pub fn invalidate_extension_info(&self) {
        self.call(nautilus_file_info_invalidate_extension_info)
    }

    /// Returns the activation URI Nautilus would open.
    pub fn activation_uri(&self) -> Option<String> {
        self.owned_string(nautilus_file_info_get_activation_uri)
    }

    /// Returns the Gio file type.
    pub fn file_type(&self) -> GFileType {
        self.value(nautilus_file_info_get_file_type)
    }

    /// Returns the Gio location object.
    pub fn location(&self) -> Option<OwnedGObject<GFile>> {
        // SAFETY: the getter returns a full-transfer GFile.
        unsafe { OwnedGObject::from_raw_full(self.pointer(nautilus_file_info_get_location)) }
    }

    /// Returns the URI for the Gio location object.
    pub fn location_uri(&self) -> Option<String> {
        let location = self.location()?;
        unsafe { take_glib_string(g_file_get_uri(location.as_ptr())) }
    }

    /// Returns the local filesystem path for the Gio location object.
    pub fn location_path(&self) -> Option<PathBuf> {
        let location = self.location()?;
        unsafe { take_glib_string(g_file_get_path(location.as_ptr())).map(PathBuf::from) }
    }

    /// Returns the Gio location of the parent directory.
    pub fn parent_location(&self) -> Option<OwnedGObject<GFile>> {
        // SAFETY: the getter returns a full-transfer GFile.
        unsafe { OwnedGObject::from_raw_full(self.pointer(nautilus_file_info_get_parent_location)) }
    }

    /// Returns the parent directory's file info.
    pub fn parent_info(&self) -> Option<FileInfo> {
        // SAFETY: the getter returns a full-transfer NautilusFileInfo.
        unsafe { FileInfo::from_raw_full(self.pointer(nautilus_file_info_get_parent_info)) }
    }

    /// Returns the mount that contains this file.
    pub fn mount(&self) -> Option<OwnedGObject<GMount>> {
        // SAFETY: the getter returns a full-transfer GMount.
        unsafe { OwnedGObject::from_raw_full(self.pointer(nautilus_file_info_get_mount)) }
    }

    /// Returns whether the current user can write to the file.
    pub fn can_write(&self) -> bool {
        self.flag(nautilus_file_info_can_write)
    }

    /// Returns the file URI or an empty string if it is unavailable.
    pub fn get_uri(&self) -> String {
        self.uri().unwrap_or_default()
    }

    /// Returns the file URI or an empty string if it is unavailable.
    ///
    /// This is the Rust-style alias for [`FileInfo::get_uri`].
    pub fn uri_or_empty(&self) -> String {
        self.get_uri()
    }

    /// Returns the URI scheme or an empty string if it is unavailable.
    pub fn get_uri_scheme(&self) -> String {
        self.uri_scheme().unwrap_or_default()
    }

    /// Returns the URI scheme or an empty string if it is unavailable.
    ///
    /// This is the Rust-style alias for [`FileInfo::get_uri_scheme`].
    pub fn uri_scheme_or_empty(&self) -> String {
        self.get_uri_scheme()
    }
}

#[derive(Debug)]
/// Owned full-transfer list of `NautilusFileInfo` objects.
pub struct FileInfoList {
    raw: *mut GList,
}

impl FileInfoList {
    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer `GList` containing
    /// `NautilusFileInfo` pointers. On success, the returned wrapper owns the
    /// list and will release it with `nautilus_file_info_list_free`.
    pub unsafe fn from_raw_full(raw: *mut GList) -> Option<FileInfoList> {
        if raw.is_null() {
            None
        } else {
            Some(FileInfoList { raw })
        }
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid borrowed `GList` containing
    /// `NautilusFileInfo` pointers.
    pub unsafe fn copy_from_raw(raw: *mut GList) -> Option<FileInfoList> {
        unsafe { FileInfoList::from_raw_full(nautilus_file_info_list_copy(raw)) }
    }

    /// Copies a slice of file-info objects into a Nautilus-owned list wrapper.
    pub fn copy(files: &[FileInfo]) -> Option<FileInfoList> {
        let mut raw_files: *mut GList = ptr::null_mut();

        for file in files {
            unsafe {
                raw_files = g_list_append(raw_files, file.raw() as gpointer);
            }
        }

        let copied = unsafe { nautilus_file_info_list_copy(raw_files) };

        unsafe {
            g_list_free(raw_files);
            FileInfoList::from_raw_full(copied)
        }
    }

    /// Returns the wrapped raw `GList` pointer.
    pub fn raw(&self) -> *mut GList {
        self.raw
    }

    /// Returns the wrapped raw `GList` pointer.
    pub fn as_ptr(&self) -> *mut GList {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw list pointer.
    pub fn into_raw(mut self) -> *mut GList {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    /// Returns the file-info objects contained in the list.
    pub fn files(&self) -> Vec<FileInfo> {
        file_info_vec_from_g_list(self.raw)
    }
}

impl Drop for FileInfoList {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                nautilus_file_info_list_free(self.raw);
            }
        }
    }
}

// SAFETY: `raw_file_info` is null or a `NautilusFileInfo` this wrapper owns a
// reference to, so it stays live for as long as the wrapper is borrowed.
unsafe impl NautilusObject for FileInfo {
    type Raw = NautilusFileInfo;

    fn as_raw(&self) -> *mut NautilusFileInfo {
        self.raw_file_info
    }
}

impl Clone for FileInfo {
    fn clone(&self) -> FileInfo {
        unsafe {
            g_object_ref(self.raw_file_info as *mut GObject);
        }
        FileInfo {
            raw_file_info: self.raw_file_info,
        }
    }
}

impl Drop for FileInfo {
    fn drop(&mut self) {
        if self.raw_file_info.is_null() {
            return;
        }

        unsafe {
            g_object_unref(self.raw_file_info as *mut GObject);
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// Borrowed handle passed to custom [`FileInfoImpl`] callbacks.
///
/// Most Nautilus extensions only consume [`FileInfo`] values supplied by
/// provider callbacks. Implement this interface only when you need to expose a
/// custom GObject that also satisfies Nautilus' `FileInfo` contract.
pub struct FileInfoHandle {
    pub(crate) raw_file_info: *mut NautilusFileInfo,
}

impl FileInfoHandle {
    /// Returns the registered `NautilusFileInfo` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_file_info_get_type() }
    }

    /// # Safety
    ///
    /// `raw_file_info` must be null or a valid borrowed `NautilusFileInfo`
    /// pointer for the duration of the callback that received it. The returned
    /// handle does not own or extend the lifetime of that pointer.
    pub unsafe fn from_raw(raw_file_info: *mut NautilusFileInfo) -> Option<FileInfoHandle> {
        if raw_file_info.is_null() {
            None
        } else {
            Some(FileInfoHandle { raw_file_info })
        }
    }

    /// Returns the borrowed raw `NautilusFileInfo` pointer.
    pub fn raw(&self) -> *mut NautilusFileInfo {
        self.raw_file_info
    }

    /// Returns the borrowed raw `NautilusFileInfo` pointer.
    pub fn as_ptr(&self) -> *mut NautilusFileInfo {
        self.raw_file_info
    }

    /// Converts this borrowed handle into an owned [`FileInfo`] reference.
    pub fn to_owned(&self) -> Option<FileInfo> {
        unsafe { FileInfo::from_raw_borrowed(self.raw_file_info) }
    }
}

/// Implements the Nautilus `FileInfo` interface on a Rust-backed object.
///
/// Extension providers normally receive Nautilus-owned [`FileInfo`] wrappers
/// and do not need this trait. It is exposed for completeness when an extension
/// module registers its own GObject type that should implement
/// `NautilusFileInfo`.
pub trait FileInfoImpl: Send + Sync {
    /// Returns whether Nautilus should treat the file as gone.
    fn is_gone(&self, _file_info: FileInfoHandle) -> bool {
        false
    }

    /// Returns the display name for the file.
    fn name(&self, _file_info: FileInfoHandle) -> Option<String> {
        None
    }

    /// Returns the file URI.
    fn uri(&self, _file_info: FileInfoHandle) -> Option<String> {
        None
    }

    /// Returns the parent directory URI.
    fn parent_uri(&self, _file_info: FileInfoHandle) -> Option<String> {
        None
    }

    /// Returns the URI scheme, such as `file` or `trash`.
    fn uri_scheme(&self, _file_info: FileInfoHandle) -> Option<String> {
        None
    }

    /// Returns the MIME type for the file.
    fn mime_type(&self, _file_info: FileInfoHandle) -> Option<String> {
        None
    }

    /// Returns whether the file matches `mime_type`.
    fn is_mime_type(&self, file_info: FileInfoHandle, mime_type: &str) -> bool {
        self.mime_type(file_info).as_deref() == Some(mime_type)
    }

    /// Returns whether the file is a directory.
    fn is_directory(&self, _file_info: FileInfoHandle) -> bool {
        false
    }

    /// Adds an emblem by icon name.
    fn add_emblem(&self, _file_info: FileInfoHandle, _emblem_name: &str) {}

    /// Returns a custom string attribute.
    fn string_attribute(
        &self,
        _file_info: FileInfoHandle,
        _attribute_name: &str,
    ) -> Option<String> {
        None
    }

    /// Adds or updates a custom string attribute.
    fn add_string_attribute(
        &self,
        _file_info: FileInfoHandle,
        _attribute_name: &str,
        _value: &str,
    ) {
    }

    /// Invalidates extension-provided information for this file.
    fn invalidate_extension_info(&self, _file_info: FileInfoHandle) {}

    /// Returns the activation URI Nautilus should open.
    fn activation_uri(&self, file_info: FileInfoHandle) -> Option<String> {
        self.uri(file_info)
    }

    /// Returns the Gio file type.
    fn file_type(&self, _file_info: FileInfoHandle) -> GFileType {
        G_FILE_TYPE_UNKNOWN
    }

    /// Returns the Gio location object.
    fn location(&self, _file_info: FileInfoHandle) -> Option<OwnedGObject<GFile>> {
        None
    }

    /// Returns the Gio location of the parent directory.
    fn parent_location(&self, _file_info: FileInfoHandle) -> Option<OwnedGObject<GFile>> {
        None
    }

    /// Returns the parent directory's file info.
    fn parent_info(&self, _file_info: FileInfoHandle) -> Option<FileInfo> {
        None
    }

    /// Returns the mount that contains this file.
    fn mount(&self, _file_info: FileInfoHandle) -> Option<OwnedGObject<GMount>> {
        None
    }

    /// Returns whether the current user can write to the file.
    fn can_write(&self, _file_info: FileInfoHandle) -> bool {
        false
    }
}

#[derive(Debug)]
/// Owned reference to a generic GObject-derived value returned by GLib/Gio.
pub struct OwnedGObject<T> {
    raw: *mut T,
    _marker: PhantomData<T>,
}

impl<T> OwnedGObject<T> {
    /// # Safety
    ///
    /// `raw` must be a valid borrowed GObject-derived pointer of type `T`.
    /// This function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(raw: *mut T) -> Option<OwnedGObject<T>> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut GObject);
        }
        Some(OwnedGObject {
            raw,
            _marker: PhantomData,
        })
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer GObject-derived
    /// pointer of type `T`. On success, the returned wrapper owns that
    /// reference.
    pub unsafe fn from_raw_full(raw: *mut T) -> Option<OwnedGObject<T>> {
        if raw.is_null() {
            return None;
        }

        Some(OwnedGObject {
            raw,
            _marker: PhantomData,
        })
    }

    /// Returns the wrapped raw GObject-derived pointer.
    pub fn as_ptr(&self) -> *mut T {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut T {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }
}

impl<T> Clone for OwnedGObject<T> {
    fn clone(&self) -> OwnedGObject<T> {
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        OwnedGObject {
            raw: self.raw,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for OwnedGObject<T> {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }

        unsafe {
            g_object_unref(self.raw as *mut GObject);
        }
    }
}
