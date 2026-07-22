use crate::gio_ffi::{
    g_file_get_path, g_file_get_uri, GFile, GFileType, GMount, G_FILE_TYPE_UNKNOWN,
};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::glib_ffi::{
    g_idle_source_new, g_source_attach, g_source_set_callback, g_source_set_priority,
    g_source_unref, G_PRIORITY_DEFAULT,
};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::glib_ffi::{g_list_append, g_list_free};
use crate::glib_ffi::{
    g_main_context_default, g_main_context_ref, g_main_context_ref_thread_default,
    g_main_context_unref, g_strdup, gboolean, gpointer, GList, GMainContext, GType, GFALSE, GTRUE,
};
use crate::gobject_ffi::{
    g_closure_ref, g_closure_unref, g_object_ref, g_object_unref, GClosure, GObject,
};
#[cfg(not(nautilus_extension_rs_skip_link))]
use crate::nautilus_ffi::{
    nautilus_file_info_add_emblem, nautilus_file_info_add_string_attribute,
    nautilus_file_info_can_write, nautilus_file_info_create, nautilus_file_info_create_for_uri,
    nautilus_file_info_get_activation_uri, nautilus_file_info_get_file_type,
    nautilus_file_info_get_location, nautilus_file_info_get_mime_type,
    nautilus_file_info_get_mount, nautilus_file_info_get_name, nautilus_file_info_get_parent_info,
    nautilus_file_info_get_parent_location, nautilus_file_info_get_parent_uri,
    nautilus_file_info_get_string_attribute, nautilus_file_info_get_type,
    nautilus_file_info_get_uri, nautilus_file_info_get_uri_scheme,
    nautilus_file_info_invalidate_extension_info, nautilus_file_info_is_directory,
    nautilus_file_info_is_gone, nautilus_file_info_is_mime_type, nautilus_file_info_list_copy,
    nautilus_file_info_list_free, nautilus_file_info_lookup, nautilus_file_info_lookup_for_uri,
    nautilus_info_provider_cancel_update, nautilus_info_provider_get_type,
    nautilus_info_provider_update_complete_invoke, nautilus_info_provider_update_file_info,
    nautilus_operation_result_get_type,
};
use crate::nautilus_ffi::{
    NautilusFileInfo, NautilusFileInfoInterface, NautilusInfoProvider, NautilusInfoProviderIface,
    NautilusOperationHandle, NautilusOperationResult,
};
use crate::slot_allocator::{release_slot, reset_slots, take_next_slot};
use crate::translate::{file_info_vec_from_g_list, take_glib_string};
use libc::c_char;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Result returned from Nautilus extension operations.
pub enum OperationResult {
    /// The operation completed successfully.
    Complete,
    /// The operation failed.
    Failed,
    /// The operation will complete asynchronously.
    InProgress,
}

impl OperationResult {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusOperationResult` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_operation_result_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusOperationResult` GType.
    pub fn type_() -> GType {
        0
    }
}

impl From<OperationResult> for NautilusOperationResult {
    fn from(result: OperationResult) -> NautilusOperationResult {
        match result {
            OperationResult::Complete => NautilusOperationResult::NautilusOperationComplete,
            OperationResult::Failed => NautilusOperationResult::NautilusOperationFailed,
            OperationResult::InProgress => NautilusOperationResult::NautilusOperationInProgress,
        }
    }
}

impl From<NautilusOperationResult> for OperationResult {
    fn from(result: NautilusOperationResult) -> OperationResult {
        match result {
            NautilusOperationResult::NautilusOperationComplete => OperationResult::Complete,
            NautilusOperationResult::NautilusOperationFailed => OperationResult::Failed,
            NautilusOperationResult::NautilusOperationInProgress => OperationResult::InProgress,
        }
    }
}

/// Supplies additional attributes and emblems for Nautilus files.
///
/// Implement `update_file_info` for quick synchronous work, or override
/// `update_file_info_full` to return [`OperationResult::InProgress`] and
/// complete the work later through [`PendingUpdate`]. `FileInfo` values are
/// main-context objects; use [`UpdateCompletion::complete_with`] to apply
/// async results to the file before Nautilus is notified. Use a bounded worker
/// queue or downstream runtime for production work; Nautilus may ask for
/// metadata for many files in one directory view.
///
/// # Synchronous Example
///
/// ```no_run
/// use nautilus_extension::{FileInfo, InfoProvider};
///
/// struct Provider;
///
/// impl InfoProvider for Provider {
///     fn update_file_info(&self, file: &mut FileInfo) {
///         let value = file.uri_scheme().unwrap_or_else(|| "unknown".to_string());
///         file.add_string_attribute("example_status", &value);
///     }
/// }
/// ```
///
/// # Asynchronous Example
///
/// ```no_run
/// use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
/// use nautilus_extension::{
///     InfoProvider, OperationHandle, OperationResult, UpdateCompletion,
///     UpdateFileInfoOperation,
/// };
///
/// const ATTRIBUTE: &str = "example_status";
///
/// struct WorkItem {
///     name: String,
///     handle: OperationHandle,
///     completion: UpdateCompletion,
/// }
///
/// struct Provider {
///     sender: SyncSender<WorkItem>,
/// }
///
/// impl Provider {
///     fn new() -> Provider {
///         let (sender, receiver) = sync_channel::<WorkItem>(32);
///
///         std::thread::spawn(move || {
///             while let Ok(work) = receiver.recv() {
///                 if work.handle.is_cancelled() {
///                     let _ = work.completion.complete(OperationResult::Failed);
///                     continue;
///                 }
///
///                 let value = format!("{} characters", work.name.chars().count());
///                 let _ = work.completion.complete_with(OperationResult::Complete, move |file| {
///                     file.add_string_attribute(ATTRIBUTE, &value);
///                 });
///             }
///         });
///
///         Provider { sender }
///     }
/// }
///
/// impl InfoProvider for Provider {
///     fn update_file_info_full(&self, operation: UpdateFileInfoOperation) -> OperationResult {
///         let name = operation.file_info().name().unwrap_or_default();
///         let pending = operation.into_pending();
///         let item = WorkItem {
///             name,
///             handle: pending.handle().clone(),
///             completion: pending.completion(),
///         };
///
///         match self.sender.try_send(item) {
///             Ok(()) => OperationResult::InProgress,
///             Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
///                 OperationResult::Failed
///             }
///         }
///     }
/// }
/// ```
pub trait InfoProvider: Send + Sync {
    /// Updates a file synchronously.
    ///
    /// Override this for quick work. For expensive work, override
    /// [`InfoProvider::update_file_info_full`] instead.
    fn update_file_info(&self, _file_info: &mut FileInfo) {}

    /// Updates a file and optionally starts an asynchronous operation.
    ///
    /// The default implementation calls [`InfoProvider::update_file_info`] and
    /// returns [`OperationResult::Complete`].
    fn update_file_info_full(&self, mut operation: UpdateFileInfoOperation) -> OperationResult {
        self.update_file_info(operation.file_info_mut());
        OperationResult::Complete
    }

    /// Cancels an asynchronous update previously returned as in progress.
    fn cancel_update(&self, _handle: &OperationHandle) {}
}

#[derive(Debug)]
/// Owned handle for a `NautilusInfoProvider` instance.
pub struct InfoProviderHandle {
    raw: *mut NautilusInfoProvider,
}

impl InfoProviderHandle {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusInfoProvider` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_info_provider_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusInfoProvider` GType.
    pub fn type_() -> GType {
        0
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer
    /// `NautilusInfoProvider` GObject pointer. On success, the returned wrapper
    /// owns that reference.
    pub unsafe fn from_raw_full(raw: *mut NautilusInfoProvider) -> Option<InfoProviderHandle> {
        if raw.is_null() {
            return None;
        }

        Some(InfoProviderHandle { raw })
    }

    /// # Safety
    ///
    /// `raw` must be a valid borrowed `NautilusInfoProvider` GObject pointer.
    /// This function adds one reference and returns an owned wrapper for that
    /// reference.
    pub unsafe fn from_raw_borrowed(raw: *mut NautilusInfoProvider) -> Option<InfoProviderHandle> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_object_ref(raw as *mut GObject);
        }
        Some(InfoProviderHandle { raw })
    }

    /// Returns the wrapped raw `NautilusInfoProvider` pointer.
    pub fn raw(&self) -> *mut NautilusInfoProvider {
        self.raw
    }

    /// Returns the wrapped raw `NautilusInfoProvider` pointer.
    pub fn as_ptr(&self) -> *mut NautilusInfoProvider {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw pointer.
    pub fn into_raw(mut self) -> *mut NautilusInfoProvider {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Invokes the raw Nautilus info-provider update function.
    ///
    /// # Safety
    ///
    /// `update_complete` must be a valid `GClosure` accepted by
    /// `nautilus_info_provider_update_complete_invoke` for the target
    /// provider, and it must remain valid according to the Nautilus extension
    /// API contract. If the returned result is `InProgress`, Nautilus owns the
    /// returned raw handle and is responsible for passing it back to
    /// `cancel_update` or the completion closure.
    pub unsafe fn update_file_info_raw(
        &self,
        file: &FileInfo,
        update_complete: *mut GClosure,
    ) -> (OperationResult, Option<OperationHandle>) {
        let mut raw_handle: *mut NautilusOperationHandle = ptr::null_mut();
        let result = unsafe {
            nautilus_info_provider_update_file_info(
                self.raw,
                file.raw(),
                update_complete,
                &mut raw_handle,
            )
        };
        let result = result.into();
        let handle = operation_handle_for_update_result(result, raw_handle);

        (result, handle)
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Invokes the raw Nautilus info-provider update function.
    ///
    /// # Safety
    ///
    /// This no-link test-mode implementation does not touch the raw closure.
    pub unsafe fn update_file_info_raw(
        &self,
        _file: &FileInfo,
        _update_complete: *mut GClosure,
    ) -> (OperationResult, Option<OperationHandle>) {
        (OperationResult::Failed, None)
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Invokes the raw Nautilus info-provider update function with a wrapped callback.
    ///
    /// # Safety
    ///
    /// `update_complete` must have been supplied by Nautilus for this provider
    /// invocation. The callback and all raw objects it closes over must remain
    /// valid according to Nautilus' `InfoProvider::update_file_info` contract.
    pub unsafe fn update_file_info_with_callback(
        &self,
        file: &FileInfo,
        update_complete: &UpdateCompleteCallback,
    ) -> (OperationResult, Option<OperationHandle>) {
        unsafe { self.update_file_info_raw(file, update_complete.raw()) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Invokes the raw Nautilus info-provider update function with a wrapped callback.
    ///
    /// # Safety
    ///
    /// This no-link test-mode implementation does not touch the raw callback.
    pub unsafe fn update_file_info_with_callback(
        &self,
        _file: &FileInfo,
        _update_complete: &UpdateCompleteCallback,
    ) -> (OperationResult, Option<OperationHandle>) {
        (OperationResult::Failed, None)
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Calls the provider's raw cancel function for an operation handle.
    pub fn cancel_update(&self, handle: &OperationHandle) {
        let raw = handle.raw();
        if raw.is_null() {
            return;
        }

        unsafe {
            nautilus_info_provider_cancel_update(self.raw, raw);
        }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Calls the provider's raw cancel function for an operation handle.
    pub fn cancel_update(&self, _handle: &OperationHandle) {}
}

impl Clone for InfoProviderHandle {
    fn clone(&self) -> InfoProviderHandle {
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        InfoProviderHandle { raw: self.raw }
    }
}

impl Drop for InfoProviderHandle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_object_unref(self.raw as *mut GObject);
            }
        }
    }
}

#[derive(Debug)]
/// Owned reference to Nautilus' `update_complete` callback closure.
///
/// Most extensions should use [`UpdateCompletion`] from
/// [`UpdateFileInfoOperation::into_pending`]. This wrapper exposes the exact
/// `nautilus_info_provider_update_complete_invoke` API for code that needs to
/// bridge a raw Nautilus callback manually.
pub struct UpdateCompleteCallback {
    raw: *mut GClosure,
}

impl UpdateCompleteCallback {
    /// # Safety
    ///
    /// `raw` must be a valid borrowed `GClosure` supplied by Nautilus for an
    /// info-provider update. This function adds one closure reference and
    /// returns an owned wrapper for that reference.
    pub unsafe fn from_raw_borrowed(raw: *mut GClosure) -> Option<UpdateCompleteCallback> {
        if raw.is_null() {
            return None;
        }

        unsafe {
            g_closure_ref(raw);
        }
        Some(UpdateCompleteCallback { raw })
    }

    /// # Safety
    ///
    /// `raw` must be either null or a valid full-transfer `GClosure`. On
    /// success, the returned wrapper owns that reference.
    pub unsafe fn from_raw_full(raw: *mut GClosure) -> Option<UpdateCompleteCallback> {
        if raw.is_null() {
            return None;
        }

        Some(UpdateCompleteCallback { raw })
    }

    /// Returns the wrapped raw `GClosure` pointer.
    pub fn raw(&self) -> *mut GClosure {
        self.raw
    }

    /// Returns the wrapped raw `GClosure` pointer.
    pub fn as_ptr(&self) -> *mut GClosure {
        self.raw
    }

    /// Consumes the wrapper and transfers ownership of the raw closure pointer.
    pub fn into_raw(mut self) -> *mut GClosure {
        let raw = self.raw;
        self.raw = ptr::null_mut();
        raw
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Invokes Nautilus' update-complete callback.
    ///
    /// # Safety
    ///
    /// The callback must belong to `provider` and `handle`, and must only be
    /// invoked once for a Nautilus operation that previously returned
    /// [`OperationResult::InProgress`].
    pub unsafe fn invoke(
        &self,
        provider: &InfoProviderHandle,
        handle: &OperationHandle,
        result: OperationResult,
    ) -> bool {
        if self.raw.is_null() || provider.raw().is_null() || handle.raw().is_null() {
            return false;
        }

        unsafe {
            nautilus_info_provider_update_complete_invoke(
                self.raw,
                provider.raw(),
                handle.raw(),
                result.into(),
            );
        }
        true
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Invokes Nautilus' update-complete callback.
    ///
    /// # Safety
    ///
    /// This no-link test-mode implementation does not invoke the raw callback.
    pub unsafe fn invoke(
        &self,
        _provider: &InfoProviderHandle,
        _handle: &OperationHandle,
        _result: OperationResult,
    ) -> bool {
        false
    }
}

impl Clone for UpdateCompleteCallback {
    fn clone(&self) -> UpdateCompleteCallback {
        unsafe {
            g_closure_ref(self.raw);
        }

        UpdateCompleteCallback { raw: self.raw }
    }
}

impl Drop for UpdateCompleteCallback {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                g_closure_unref(self.raw);
            }
        }
    }
}

#[derive(Debug)]
/// Owned reference to a `NautilusFileInfo`.
pub struct FileInfo {
    raw_file_info: *mut NautilusFileInfo,
}

impl FileInfo {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusFileInfo` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_file_info_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusFileInfo` GType.
    pub fn type_() -> GType {
        0
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
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { FileInfo::from_raw_full(nautilus_file_info_create(location.as_ptr())) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = location;
            None
        }
    }

    /// Creates a `FileInfo` for a URI.
    pub fn create_for_uri(uri: &str) -> Option<FileInfo> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            let uri = CString::new(uri).ok()?;
            unsafe { FileInfo::from_raw_full(nautilus_file_info_create_for_uri(uri.as_ptr())) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = uri;
            None
        }
    }

    /// Looks up an existing `FileInfo` for a Gio file location.
    pub fn lookup(location: &OwnedGObject<GFile>) -> Option<FileInfo> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { FileInfo::from_raw_full(nautilus_file_info_lookup(location.as_ptr())) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = location;
            None
        }
    }

    /// Looks up an existing `FileInfo` for a URI.
    pub fn lookup_for_uri(uri: &str) -> Option<FileInfo> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            let uri = CString::new(uri).ok()?;
            unsafe { FileInfo::from_raw_full(nautilus_file_info_lookup_for_uri(uri.as_ptr())) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = uri;
            None
        }
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
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { nautilus_file_info_is_gone(self.raw_file_info) != GFALSE }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            false
        }
    }

    /// Returns the display name for the file.
    pub fn name(&self) -> Option<String> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { take_glib_string(nautilus_file_info_get_name(self.raw_file_info)) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns the file URI.
    pub fn uri(&self) -> Option<String> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { take_glib_string(nautilus_file_info_get_uri(self.raw_file_info)) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns the parent directory URI.
    pub fn parent_uri(&self) -> Option<String> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { take_glib_string(nautilus_file_info_get_parent_uri(self.raw_file_info)) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns the URI scheme, such as `file` or `trash`.
    pub fn uri_scheme(&self) -> Option<String> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { take_glib_string(nautilus_file_info_get_uri_scheme(self.raw_file_info)) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns the MIME type known by Nautilus.
    pub fn mime_type(&self) -> Option<String> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { take_glib_string(nautilus_file_info_get_mime_type(self.raw_file_info)) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns whether the file matches `mime_type`.
    pub fn is_mime_type(&self, mime_type: &str) -> bool {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            let mime_type = match CString::new(mime_type) {
                Ok(mime_type) => mime_type,
                Err(_) => return false,
            };

            unsafe {
                nautilus_file_info_is_mime_type(self.raw_file_info, mime_type.as_ptr()) != GFALSE
            }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = mime_type;
            false
        }
    }

    /// Returns whether the file is a directory.
    pub fn is_directory(&self) -> bool {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { nautilus_file_info_is_directory(self.raw_file_info) != GFALSE }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            false
        }
    }

    /// Adds an emblem by icon name.
    pub fn add_emblem(&self, emblem_name: &str) {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            let emblem_name = match CString::new(emblem_name) {
                Ok(emblem_name) => emblem_name,
                Err(_) => return,
            };

            unsafe {
                nautilus_file_info_add_emblem(self.raw_file_info, emblem_name.as_ptr());
            }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = emblem_name;
        }
    }

    /// Returns a string attribute previously known to Nautilus.
    pub fn string_attribute(&self, attribute_name: &str) -> Option<String> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            let attribute_name = CString::new(attribute_name).ok()?;
            unsafe {
                take_glib_string(nautilus_file_info_get_string_attribute(
                    self.raw_file_info,
                    attribute_name.as_ptr(),
                ))
            }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = attribute_name;
            None
        }
    }

    /// Adds or updates a string attribute for this file.
    pub fn add_string_attribute(&self, attribute_name: &str, value: &str) {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
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

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            let _ = (attribute_name, value);
        }
    }

    /// Alias for [`FileInfo::add_string_attribute`].
    pub fn add_attribute(&self, attribute_name: &str, value: &str) {
        self.add_string_attribute(attribute_name, value);
    }

    /// Asks Nautilus to refresh extension-provided info for this file.
    pub fn invalidate_extension_info(&self) {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe {
                nautilus_file_info_invalidate_extension_info(self.raw_file_info);
            }
        }
    }

    /// Returns the activation URI Nautilus would open.
    pub fn activation_uri(&self) -> Option<String> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { take_glib_string(nautilus_file_info_get_activation_uri(self.raw_file_info)) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns the Gio file type.
    pub fn file_type(&self) -> GFileType {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { nautilus_file_info_get_file_type(self.raw_file_info) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            G_FILE_TYPE_UNKNOWN
        }
    }

    /// Returns the Gio location object.
    pub fn location(&self) -> Option<OwnedGObject<GFile>> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe {
                OwnedGObject::from_raw_full(nautilus_file_info_get_location(self.raw_file_info))
            }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
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
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe {
                OwnedGObject::from_raw_full(nautilus_file_info_get_parent_location(
                    self.raw_file_info,
                ))
            }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns the parent directory's file info.
    pub fn parent_info(&self) -> Option<FileInfo> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe {
                FileInfo::from_raw_full(nautilus_file_info_get_parent_info(self.raw_file_info))
            }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns the mount that contains this file.
    pub fn mount(&self) -> Option<OwnedGObject<GMount>> {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { OwnedGObject::from_raw_full(nautilus_file_info_get_mount(self.raw_file_info)) }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            None
        }
    }

    /// Returns whether the current user can write to the file.
    pub fn can_write(&self) -> bool {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        {
            unsafe { nautilus_file_info_can_write(self.raw_file_info) != GFALSE }
        }

        #[cfg(nautilus_extension_rs_skip_link)]
        {
            false
        }
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

    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// # Safety
    ///
    /// `raw` must be either null or a valid borrowed `GList` containing
    /// `NautilusFileInfo` pointers.
    pub unsafe fn copy_from_raw(raw: *mut GList) -> Option<FileInfoList> {
        unsafe { FileInfoList::from_raw_full(nautilus_file_info_list_copy(raw)) }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// # Safety
    ///
    /// This no-link test-mode implementation does not inspect `raw`.
    pub unsafe fn copy_from_raw(_raw: *mut GList) -> Option<FileInfoList> {
        None
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
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

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Copies a slice of file-info objects into a Nautilus-owned list wrapper.
    pub fn copy(_files: &[FileInfo]) -> Option<FileInfoList> {
        None
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
            #[cfg(not(nautilus_extension_rs_skip_link))]
            unsafe {
                nautilus_file_info_list_free(self.raw);
            }
        }
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
    raw_file_info: *mut NautilusFileInfo,
}

impl FileInfoHandle {
    #[cfg(not(nautilus_extension_rs_skip_link))]
    /// Returns the registered `NautilusFileInfo` GType.
    pub fn type_() -> GType {
        unsafe { nautilus_file_info_get_type() }
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    /// Returns the registered `NautilusFileInfo` GType.
    pub fn type_() -> GType {
        0
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

#[derive(Clone)]
/// Opaque handle used to identify an asynchronous InfoProvider operation.
pub struct OperationHandle {
    raw: usize,
    state: Option<Arc<OperationState>>,
}

impl OperationHandle {
    /// Returns an opaque stable identifier for this operation handle.
    pub fn id(&self) -> usize {
        self.raw
    }

    /// Returns the raw `NautilusOperationHandle` pointer.
    pub fn raw(&self) -> *mut NautilusOperationHandle {
        self.raw as *mut NautilusOperationHandle
    }

    /// Returns whether Nautilus has requested cancellation for this operation.
    pub fn is_cancelled(&self) -> bool {
        self.state
            .as_ref()
            .map(|state| state.cancelled.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    fn new(raw: *mut NautilusOperationHandle, state: Arc<OperationState>) -> OperationHandle {
        OperationHandle {
            raw: raw as usize,
            state: Some(state),
        }
    }

    fn unknown(raw: *mut NautilusOperationHandle) -> OperationHandle {
        OperationHandle {
            raw: raw as usize,
            state: None,
        }
    }
}

/// Completion callback for an asynchronous InfoProvider operation.
pub struct UpdateCompletion {
    state: Arc<OperationState>,
}

impl Clone for UpdateCompletion {
    fn clone(&self) -> UpdateCompletion {
        UpdateCompletion {
            state: self.state.clone(),
        }
    }
}

impl UpdateCompletion {
    /// Completes an async update on Nautilus' main context without changing the file info.
    pub fn complete(&self, result: OperationResult) -> bool {
        self.complete_with(result, |_| {})
    }

    /// Applies `update_file_info` on Nautilus' main context, then completes the operation.
    ///
    /// Returns `false` if the operation was already completed or the completion
    /// could not be scheduled.
    pub fn complete_with<F>(&self, result: OperationResult, update_file_info: F) -> bool
    where
        F: FnOnce(&mut FileInfo) + Send + 'static,
    {
        if self
            .state
            .completed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return false;
        }

        let scheduled = schedule_completion_source(CompletionSource {
            state: self.state.clone(),
            result,
            update_file_info: Some(Box::new(update_file_info)),
        });

        if !scheduled {
            self.state.completed.store(false, Ordering::SeqCst);
        }

        scheduled
    }
}

/// Pending asynchronous update returned by [`UpdateFileInfoOperation::into_pending`].
pub struct PendingUpdate {
    handle: OperationHandle,
    completion: UpdateCompletion,
}

impl PendingUpdate {
    /// Returns the operation handle used for cancellation checks.
    pub fn handle(&self) -> &OperationHandle {
        &self.handle
    }

    /// Returns a completion object that can finish the pending update.
    pub fn completion(&self) -> UpdateCompletion {
        self.completion.clone()
    }
}

/// Arguments passed to `InfoProvider::update_file_info_full`.
pub struct UpdateFileInfoOperation {
    file_info: FileInfo,
    handle: OperationHandle,
    completion: UpdateCompletion,
}

impl UpdateFileInfoOperation {
    /// Returns the file being updated.
    pub fn file_info(&self) -> &FileInfo {
        &self.file_info
    }

    /// Returns the file being updated.
    pub fn file_info_mut(&mut self) -> &mut FileInfo {
        &mut self.file_info
    }

    /// Consumes the operation and returns the file being updated.
    pub fn into_file_info(self) -> FileInfo {
        self.file_info
    }

    /// Returns the operation handle used for cancellation checks.
    pub fn handle(&self) -> &OperationHandle {
        &self.handle
    }

    /// Consumes the operation and returns the pieces needed to finish it later.
    ///
    /// Call this only when `update_file_info_full` will return
    /// [`OperationResult::InProgress`].
    pub fn into_pending(self) -> PendingUpdate {
        PendingUpdate {
            handle: self.handle,
            completion: self.completion,
        }
    }
}

struct OperationState {
    provider: usize,
    file_info: usize,
    closure: usize,
    main_context: usize,
    raw_handle: AtomicUsize,
    accepted: AtomicBool,
    cancelled: AtomicBool,
    completed: AtomicBool,
    cleaned: AtomicBool,
    cleanup_lock: Mutex<()>,
}

impl OperationState {
    fn new(
        provider: *mut NautilusInfoProvider,
        file_info: *mut NautilusFileInfo,
        closure: *mut GClosure,
    ) -> OperationState {
        if !provider.is_null() {
            unsafe {
                g_object_ref(provider as *mut GObject);
            }
        }

        if !file_info.is_null() {
            unsafe {
                g_object_ref(file_info as *mut GObject);
            }
        }

        let main_context = unsafe {
            let context = g_main_context_ref_thread_default();
            if context.is_null() {
                g_main_context_ref(g_main_context_default())
            } else {
                context
            }
        };

        OperationState {
            provider: provider as usize,
            file_info: file_info as usize,
            closure: closure as usize,
            main_context: main_context as usize,
            raw_handle: AtomicUsize::new(0),
            accepted: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            completed: AtomicBool::new(false),
            cleaned: AtomicBool::new(false),
            cleanup_lock: Mutex::new(()),
        }
    }

    fn provider(&self) -> *mut NautilusInfoProvider {
        self.provider as *mut NautilusInfoProvider
    }

    fn file_info(&self) -> *mut NautilusFileInfo {
        self.file_info as *mut NautilusFileInfo
    }

    fn closure(&self) -> *mut GClosure {
        self.closure as *mut GClosure
    }

    fn main_context(&self) -> *mut GMainContext {
        self.main_context as *mut GMainContext
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    fn raw_handle(&self) -> *mut NautilusOperationHandle {
        self.raw_handle.load(Ordering::SeqCst) as *mut NautilusOperationHandle
    }
}

type UpdateFileInfoFn = Box<dyn FnOnce(&mut FileInfo) + Send + 'static>;

#[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
struct CompletionSource {
    state: Arc<OperationState>,
    result: OperationResult,
    update_file_info: Option<UpdateFileInfoFn>,
}

#[cfg(not(nautilus_extension_rs_skip_link))]
unsafe extern "C" fn run_completion_source(data: gpointer) -> gboolean {
    if data.is_null() {
        return GFALSE;
    }

    let source = unsafe { &mut *(data as *mut CompletionSource) };

    if source.state.cleaned.load(Ordering::SeqCst) {
        return GFALSE;
    }

    if !source.state.accepted.load(Ordering::SeqCst) {
        return GTRUE;
    }

    let raw = source.state.raw_handle();
    let was_in_flight = finish_in_flight(raw);

    if !was_in_flight || source.state.cancelled.load(Ordering::SeqCst) {
        cleanup_state(&source.state);
        return GFALSE;
    }

    let mut result = source.result;

    if let Some(update_file_info) = source.update_file_info.take() {
        let update_result = catch_unwind(AssertUnwindSafe(|| {
            if let Some(mut file_info) =
                unsafe { FileInfo::from_raw_borrowed(source.state.file_info()) }
            {
                update_file_info(&mut file_info);
            } else {
                result = OperationResult::Failed;
            }
        }));

        if update_result.is_err() {
            result = OperationResult::Failed;
        }
    }

    unsafe {
        nautilus_info_provider_update_complete_invoke(
            source.state.closure(),
            source.state.provider(),
            raw,
            result.into(),
        );
    }

    cleanup_state(&source.state);
    GFALSE
}

#[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
unsafe extern "C" fn destroy_completion_source(data: gpointer) {
    if !data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(unsafe { Box::from_raw(data as *mut CompletionSource) });
        }));
    }
}

#[cfg(not(nautilus_extension_rs_skip_link))]
fn ref_main_context_for_source(state: &OperationState) -> Option<*mut GMainContext> {
    let _guard = state.cleanup_lock.lock().ok()?;

    if state.cleaned.load(Ordering::SeqCst) {
        return None;
    }

    let context = state.main_context();
    if context.is_null() {
        None
    } else {
        Some(unsafe { g_main_context_ref(context) })
    }
}

#[cfg(not(nautilus_extension_rs_skip_link))]
fn schedule_completion_source(source_data: CompletionSource) -> bool {
    let context = match ref_main_context_for_source(&source_data.state) {
        Some(context) => context,
        None => return false,
    };

    let source = unsafe { g_idle_source_new() };
    if source.is_null() {
        unsafe {
            g_main_context_unref(context);
        }
        return false;
    }

    let source_data = Box::into_raw(Box::new(source_data));

    unsafe {
        g_source_set_priority(source, G_PRIORITY_DEFAULT);
        g_source_set_callback(
            source,
            Some(run_completion_source),
            source_data as gpointer,
            Some(destroy_completion_source),
        );
        g_source_attach(source, context);
        g_source_unref(source);
        g_main_context_unref(context);
    }

    true
}

#[cfg(nautilus_extension_rs_skip_link)]
fn schedule_completion_source(_source_data: CompletionSource) -> bool {
    false
}

lazy_static! {
    static ref IN_FLIGHT_OPERATIONS: Mutex<HashMap<usize, Arc<OperationState>>> =
        Mutex::new(HashMap::new());
}

fn register_in_flight(raw: *mut NautilusOperationHandle, state: Arc<OperationState>) -> bool {
    match IN_FLIGHT_OPERATIONS.lock() {
        Ok(mut operations) => {
            operations.insert(raw as usize, state);
            true
        }
        Err(_) => false,
    }
}

fn lookup_handle(raw: *mut NautilusOperationHandle) -> OperationHandle {
    let state = IN_FLIGHT_OPERATIONS
        .lock()
        .ok()
        .and_then(|operations| operations.get(&(raw as usize)).cloned());

    match state {
        Some(state) => OperationHandle::new(raw, state),
        None => OperationHandle::unknown(raw),
    }
}

fn finish_in_flight(raw: *mut NautilusOperationHandle) -> bool {
    let removed = IN_FLIGHT_OPERATIONS
        .lock()
        .ok()
        .and_then(|mut operations| operations.remove(&(raw as usize)));

    if removed.is_some() {
        drop_raw_operation_handle(raw);
        true
    } else {
        false
    }
}

fn drop_raw_operation_handle(raw: *mut NautilusOperationHandle) {
    if !raw.is_null() {
        unsafe {
            drop(Arc::from_raw(raw as *const OperationState));
        }
    }
}

fn cleanup_state(state: &OperationState) {
    let _guard = match state.cleanup_lock.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    if state.cleaned.swap(true, Ordering::SeqCst) {
        return;
    }

    unsafe {
        g_closure_unref(state.closure());
        if !state.file_info().is_null() {
            g_object_unref(state.file_info() as *mut GObject);
        }
        if !state.provider().is_null() {
            g_object_unref(state.provider() as *mut GObject);
        }
        if !state.main_context().is_null() {
            g_main_context_unref(state.main_context());
        }
    }
}

fn new_operation(
    provider: *mut NautilusInfoProvider,
    file: *mut NautilusFileInfo,
    update_complete: *mut GClosure,
) -> Option<(
    UpdateFileInfoOperation,
    *mut NautilusOperationHandle,
    Arc<OperationState>,
)> {
    if update_complete.is_null() {
        return None;
    }

    let file_info = unsafe { FileInfo::from_raw_borrowed(file)? };
    let closure = unsafe { g_closure_ref(update_complete) };
    let state = Arc::new(OperationState::new(provider, file, closure));
    let raw_handle = Arc::into_raw(state.clone()) as *mut NautilusOperationHandle;
    state
        .raw_handle
        .store(raw_handle as usize, Ordering::SeqCst);

    let handle = OperationHandle::new(raw_handle, state.clone());
    let completion = UpdateCompletion {
        state: state.clone(),
    };

    Some((
        UpdateFileInfoOperation {
            file_info,
            handle,
            completion,
        },
        raw_handle,
        state,
    ))
}

fn bool_to_gboolean(value: bool) -> gboolean {
    if value {
        GTRUE
    } else {
        GFALSE
    }
}

#[cfg_attr(nautilus_extension_rs_skip_link, allow(dead_code))]
fn operation_handle_for_update_result(
    result: OperationResult,
    raw_handle: *mut NautilusOperationHandle,
) -> Option<OperationHandle> {
    if result == OperationResult::InProgress && !raw_handle.is_null() {
        Some(OperationHandle::unknown(raw_handle))
    } else {
        None
    }
}

fn dup_optional_string(value: Option<String>) -> *mut c_char {
    let value = match value.and_then(|value| CString::new(value).ok()) {
        Some(value) => value,
        None => return ptr::null_mut(),
    };

    unsafe { g_strdup(value.as_ptr()) }
}

unsafe fn c_string_arg(raw: *const c_char) -> Option<String> {
    if raw.is_null() {
        return None;
    }

    Some(
        unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned(),
    )
}

fn owned_gobject_to_raw<T>(object: Option<OwnedGObject<T>>) -> *mut T {
    object
        .map(|object| object.into_raw())
        .unwrap_or(ptr::null_mut())
}

fn file_info_to_raw(file_info: Option<FileInfo>) -> *mut NautilusFileInfo {
    file_info
        .map(|file_info| file_info.into_raw())
        .unwrap_or(ptr::null_mut())
}

fn with_file_info_impl<R, F>(
    raw_file_info: *mut NautilusFileInfo,
    rust_file_info: &Mutex<Option<Arc<dyn FileInfoImpl>>>,
    default: R,
    call: F,
) -> R
where
    F: FnOnce(Arc<dyn FileInfoImpl>, FileInfoHandle) -> R,
{
    let handle = unsafe { FileInfoHandle::from_raw(raw_file_info) };
    let rust_file_info = rust_file_info
        .lock()
        .ok()
        .and_then(|file_info| file_info.clone());

    match (rust_file_info, handle) {
        (Some(file_info), Some(handle)) => {
            catch_unwind(AssertUnwindSafe(|| call(file_info, handle))).unwrap_or(default)
        }
        _ => default,
    }
}

macro_rules! file_info_iface {
    (
        $iface_init_fn:ident,
        $is_gone_fn:ident,
        $get_name_fn:ident,
        $get_uri_fn:ident,
        $get_parent_uri_fn:ident,
        $get_uri_scheme_fn:ident,
        $get_mime_type_fn:ident,
        $is_mime_type_fn:ident,
        $is_directory_fn:ident,
        $add_emblem_fn:ident,
        $get_string_attribute_fn:ident,
        $add_string_attribute_fn:ident,
        $invalidate_extension_info_fn:ident,
        $get_activation_uri_fn:ident,
        $get_file_type_fn:ident,
        $get_location_fn:ident,
        $get_parent_location_fn:ident,
        $get_parent_info_fn:ident,
        $get_mount_fn:ident,
        $can_write_fn:ident,
        $rust_file_info:ident,
        $set_rust_file_info:ident,
        $clear_rust_file_info:ident
    ) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_file_info()` instead.
        unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusFileInfoInterface;
            unsafe {
                (*iface_struct).is_gone = Some($is_gone_fn);
                (*iface_struct).get_name = Some($get_name_fn);
                (*iface_struct).get_uri = Some($get_uri_fn);
                (*iface_struct).get_parent_uri = Some($get_parent_uri_fn);
                (*iface_struct).get_uri_scheme = Some($get_uri_scheme_fn);
                (*iface_struct).get_mime_type = Some($get_mime_type_fn);
                (*iface_struct).is_mime_type = Some($is_mime_type_fn);
                (*iface_struct).is_directory = Some($is_directory_fn);
                (*iface_struct).add_emblem = Some($add_emblem_fn);
                (*iface_struct).get_string_attribute = Some($get_string_attribute_fn);
                (*iface_struct).add_string_attribute = Some($add_string_attribute_fn);
                (*iface_struct).invalidate_extension_info = Some($invalidate_extension_info_fn);
                (*iface_struct).get_activation_uri = Some($get_activation_uri_fn);
                (*iface_struct).get_file_type = Some($get_file_type_fn);
                (*iface_struct).get_location = Some($get_location_fn);
                (*iface_struct).get_parent_location = Some($get_parent_location_fn);
                (*iface_struct).get_parent_info = Some($get_parent_info_fn);
                (*iface_struct).get_mount = Some($get_mount_fn);
                (*iface_struct).can_write = Some($can_write_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $is_gone_fn(file_info: *mut NautilusFileInfo) -> gboolean {
            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.is_gone(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_name_fn(file_info: *mut NautilusFileInfo) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.name(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_uri_fn(file_info: *mut NautilusFileInfo) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.uri(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_parent_uri_fn(file_info: *mut NautilusFileInfo) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.parent_uri(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_uri_scheme_fn(file_info: *mut NautilusFileInfo) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.uri_scheme(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_mime_type_fn(file_info: *mut NautilusFileInfo) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.mime_type(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $is_mime_type_fn(
            file_info: *mut NautilusFileInfo,
            mime_type: *const c_char,
        ) -> gboolean {
            let mime_type = match unsafe { c_string_arg(mime_type) } {
                Some(mime_type) => mime_type,
                None => return GFALSE,
            };

            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.is_mime_type(handle, &mime_type),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $is_directory_fn(file_info: *mut NautilusFileInfo) -> gboolean {
            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.is_directory(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $add_emblem_fn(
            file_info: *mut NautilusFileInfo,
            emblem_name: *const c_char,
        ) {
            let emblem_name = match unsafe { c_string_arg(emblem_name) } {
                Some(emblem_name) => emblem_name,
                None => return,
            };

            with_file_info_impl(file_info, &$rust_file_info, (), |file_info, handle| {
                file_info.add_emblem(handle, &emblem_name);
            });
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_string_attribute_fn(
            file_info: *mut NautilusFileInfo,
            attribute_name: *const c_char,
        ) -> *mut c_char {
            let attribute_name = match unsafe { c_string_arg(attribute_name) } {
                Some(attribute_name) => attribute_name,
                None => return ptr::null_mut(),
            };

            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.string_attribute(handle, &attribute_name),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $add_string_attribute_fn(
            file_info: *mut NautilusFileInfo,
            attribute_name: *const c_char,
            value: *const c_char,
        ) {
            let attribute_name = match unsafe { c_string_arg(attribute_name) } {
                Some(attribute_name) => attribute_name,
                None => return,
            };
            let value = match unsafe { c_string_arg(value) } {
                Some(value) => value,
                None => return,
            };

            with_file_info_impl(file_info, &$rust_file_info, (), |file_info, handle| {
                file_info.add_string_attribute(handle, &attribute_name, &value);
            });
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $invalidate_extension_info_fn(file_info: *mut NautilusFileInfo) {
            with_file_info_impl(file_info, &$rust_file_info, (), |file_info, handle| {
                file_info.invalidate_extension_info(handle);
            });
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_activation_uri_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut c_char {
            dup_optional_string(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.activation_uri(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_file_type_fn(file_info: *mut NautilusFileInfo) -> GFileType {
            with_file_info_impl(
                file_info,
                &$rust_file_info,
                G_FILE_TYPE_UNKNOWN,
                |file_info, handle| file_info.file_type(handle),
            )
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_location_fn(file_info: *mut NautilusFileInfo) -> *mut GFile {
            owned_gobject_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.location(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_parent_location_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut GFile {
            owned_gobject_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.parent_location(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_parent_info_fn(
            file_info: *mut NautilusFileInfo,
        ) -> *mut NautilusFileInfo {
            file_info_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.parent_info(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $get_mount_fn(file_info: *mut NautilusFileInfo) -> *mut GMount {
            owned_gobject_to_raw(with_file_info_impl(
                file_info,
                &$rust_file_info,
                None,
                |file_info, handle| file_info.mount(handle),
            ))
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $can_write_fn(file_info: *mut NautilusFileInfo) -> gboolean {
            bool_to_gboolean(with_file_info_impl(
                file_info,
                &$rust_file_info,
                false,
                |file_info, handle| file_info.can_write(handle),
            ))
        }

        fn $set_rust_file_info(file_info: Box<dyn FileInfoImpl>) {
            if let Ok(mut current_file_info) = $rust_file_info.lock() {
                *current_file_info = Some(Arc::from(file_info));
            }
        }

        fn $clear_rust_file_info() {
            if let Ok(mut file_info) = $rust_file_info.lock() {
                *file_info = None;
            }
        }

        lazy_static! {
            static ref $rust_file_info: Mutex<Option<Arc<dyn FileInfoImpl>>> = Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_FILE_INFO_IMPLS: usize = 10;

#[rustfmt::skip] file_info_iface!(file_info_iface_init_0, file_info_is_gone_0, file_info_get_name_0, file_info_get_uri_0, file_info_get_parent_uri_0, file_info_get_uri_scheme_0, file_info_get_mime_type_0, file_info_is_mime_type_0, file_info_is_directory_0, file_info_add_emblem_0, file_info_get_string_attribute_0, file_info_add_string_attribute_0, file_info_invalidate_extension_info_0, file_info_get_activation_uri_0, file_info_get_file_type_0, file_info_get_location_0, file_info_get_parent_location_0, file_info_get_parent_info_0, file_info_get_mount_0, file_info_can_write_0, FILE_INFO_IMPL_0, set_file_info_impl_0, clear_file_info_impl_0);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_1, file_info_is_gone_1, file_info_get_name_1, file_info_get_uri_1, file_info_get_parent_uri_1, file_info_get_uri_scheme_1, file_info_get_mime_type_1, file_info_is_mime_type_1, file_info_is_directory_1, file_info_add_emblem_1, file_info_get_string_attribute_1, file_info_add_string_attribute_1, file_info_invalidate_extension_info_1, file_info_get_activation_uri_1, file_info_get_file_type_1, file_info_get_location_1, file_info_get_parent_location_1, file_info_get_parent_info_1, file_info_get_mount_1, file_info_can_write_1, FILE_INFO_IMPL_1, set_file_info_impl_1, clear_file_info_impl_1);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_2, file_info_is_gone_2, file_info_get_name_2, file_info_get_uri_2, file_info_get_parent_uri_2, file_info_get_uri_scheme_2, file_info_get_mime_type_2, file_info_is_mime_type_2, file_info_is_directory_2, file_info_add_emblem_2, file_info_get_string_attribute_2, file_info_add_string_attribute_2, file_info_invalidate_extension_info_2, file_info_get_activation_uri_2, file_info_get_file_type_2, file_info_get_location_2, file_info_get_parent_location_2, file_info_get_parent_info_2, file_info_get_mount_2, file_info_can_write_2, FILE_INFO_IMPL_2, set_file_info_impl_2, clear_file_info_impl_2);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_3, file_info_is_gone_3, file_info_get_name_3, file_info_get_uri_3, file_info_get_parent_uri_3, file_info_get_uri_scheme_3, file_info_get_mime_type_3, file_info_is_mime_type_3, file_info_is_directory_3, file_info_add_emblem_3, file_info_get_string_attribute_3, file_info_add_string_attribute_3, file_info_invalidate_extension_info_3, file_info_get_activation_uri_3, file_info_get_file_type_3, file_info_get_location_3, file_info_get_parent_location_3, file_info_get_parent_info_3, file_info_get_mount_3, file_info_can_write_3, FILE_INFO_IMPL_3, set_file_info_impl_3, clear_file_info_impl_3);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_4, file_info_is_gone_4, file_info_get_name_4, file_info_get_uri_4, file_info_get_parent_uri_4, file_info_get_uri_scheme_4, file_info_get_mime_type_4, file_info_is_mime_type_4, file_info_is_directory_4, file_info_add_emblem_4, file_info_get_string_attribute_4, file_info_add_string_attribute_4, file_info_invalidate_extension_info_4, file_info_get_activation_uri_4, file_info_get_file_type_4, file_info_get_location_4, file_info_get_parent_location_4, file_info_get_parent_info_4, file_info_get_mount_4, file_info_can_write_4, FILE_INFO_IMPL_4, set_file_info_impl_4, clear_file_info_impl_4);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_5, file_info_is_gone_5, file_info_get_name_5, file_info_get_uri_5, file_info_get_parent_uri_5, file_info_get_uri_scheme_5, file_info_get_mime_type_5, file_info_is_mime_type_5, file_info_is_directory_5, file_info_add_emblem_5, file_info_get_string_attribute_5, file_info_add_string_attribute_5, file_info_invalidate_extension_info_5, file_info_get_activation_uri_5, file_info_get_file_type_5, file_info_get_location_5, file_info_get_parent_location_5, file_info_get_parent_info_5, file_info_get_mount_5, file_info_can_write_5, FILE_INFO_IMPL_5, set_file_info_impl_5, clear_file_info_impl_5);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_6, file_info_is_gone_6, file_info_get_name_6, file_info_get_uri_6, file_info_get_parent_uri_6, file_info_get_uri_scheme_6, file_info_get_mime_type_6, file_info_is_mime_type_6, file_info_is_directory_6, file_info_add_emblem_6, file_info_get_string_attribute_6, file_info_add_string_attribute_6, file_info_invalidate_extension_info_6, file_info_get_activation_uri_6, file_info_get_file_type_6, file_info_get_location_6, file_info_get_parent_location_6, file_info_get_parent_info_6, file_info_get_mount_6, file_info_can_write_6, FILE_INFO_IMPL_6, set_file_info_impl_6, clear_file_info_impl_6);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_7, file_info_is_gone_7, file_info_get_name_7, file_info_get_uri_7, file_info_get_parent_uri_7, file_info_get_uri_scheme_7, file_info_get_mime_type_7, file_info_is_mime_type_7, file_info_is_directory_7, file_info_add_emblem_7, file_info_get_string_attribute_7, file_info_add_string_attribute_7, file_info_invalidate_extension_info_7, file_info_get_activation_uri_7, file_info_get_file_type_7, file_info_get_location_7, file_info_get_parent_location_7, file_info_get_parent_info_7, file_info_get_mount_7, file_info_can_write_7, FILE_INFO_IMPL_7, set_file_info_impl_7, clear_file_info_impl_7);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_8, file_info_is_gone_8, file_info_get_name_8, file_info_get_uri_8, file_info_get_parent_uri_8, file_info_get_uri_scheme_8, file_info_get_mime_type_8, file_info_is_mime_type_8, file_info_is_directory_8, file_info_add_emblem_8, file_info_get_string_attribute_8, file_info_add_string_attribute_8, file_info_invalidate_extension_info_8, file_info_get_activation_uri_8, file_info_get_file_type_8, file_info_get_location_8, file_info_get_parent_location_8, file_info_get_parent_info_8, file_info_get_mount_8, file_info_can_write_8, FILE_INFO_IMPL_8, set_file_info_impl_8, clear_file_info_impl_8);
#[rustfmt::skip] file_info_iface!(file_info_iface_init_9, file_info_is_gone_9, file_info_get_name_9, file_info_get_uri_9, file_info_get_parent_uri_9, file_info_get_uri_scheme_9, file_info_get_mime_type_9, file_info_is_mime_type_9, file_info_is_directory_9, file_info_add_emblem_9, file_info_get_string_attribute_9, file_info_add_string_attribute_9, file_info_invalidate_extension_info_9, file_info_get_activation_uri_9, file_info_get_file_type_9, file_info_get_location_9, file_info_get_parent_location_9, file_info_get_parent_info_9, file_info_get_mount_9, file_info_can_write_9, FILE_INFO_IMPL_9, set_file_info_impl_9, clear_file_info_impl_9);

#[doc(hidden)]
pub fn file_info_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        file_info_iface_init_0,
        file_info_iface_init_1,
        file_info_iface_init_2,
        file_info_iface_init_3,
        file_info_iface_init_4,
        file_info_iface_init_5,
        file_info_iface_init_6,
        file_info_iface_init_7,
        file_info_iface_init_8,
        file_info_iface_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_file_info_impl_setters() -> Vec<fn(Box<dyn FileInfoImpl>)> {
    vec![
        set_file_info_impl_0,
        set_file_info_impl_1,
        set_file_info_impl_2,
        set_file_info_impl_3,
        set_file_info_impl_4,
        set_file_info_impl_5,
        set_file_info_impl_6,
        set_file_info_impl_7,
        set_file_info_impl_8,
        set_file_info_impl_9,
    ]
}

fn rust_file_info_impl_clearers() -> Vec<fn()> {
    vec![
        clear_file_info_impl_0,
        clear_file_info_impl_1,
        clear_file_info_impl_2,
        clear_file_info_impl_3,
        clear_file_info_impl_4,
        clear_file_info_impl_5,
        clear_file_info_impl_6,
        clear_file_info_impl_7,
        clear_file_info_impl_8,
        clear_file_info_impl_9,
    ]
}

static RESERVED_FILE_INFO_IFACE_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_file_info_iface_index() -> Option<usize> {
    take_next_slot(&RESERVED_FILE_INFO_IFACE_SLOTS, MAX_FILE_INFO_IMPLS)
}

pub(crate) fn release_file_info_iface_index(index: usize) {
    if let Some(clear_file_info) = rust_file_info_impl_clearers().get(index) {
        clear_file_info();
    }

    release_slot(&RESERVED_FILE_INFO_IFACE_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn file_info_impl_slot_is_set(index: usize) -> bool {
    match index {
        0 => FILE_INFO_IMPL_0
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        1 => FILE_INFO_IMPL_1
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        2 => FILE_INFO_IMPL_2
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        3 => FILE_INFO_IMPL_3
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        4 => FILE_INFO_IMPL_4
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        5 => FILE_INFO_IMPL_5
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        6 => FILE_INFO_IMPL_6
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        7 => FILE_INFO_IMPL_7
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        8 => FILE_INFO_IMPL_8
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        9 => FILE_INFO_IMPL_9
            .lock()
            .map(|file_info| file_info.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_file_info_impl_state() {
    reset_slots(&RESERVED_FILE_INFO_IFACE_SLOTS);
    for clear_file_info in rust_file_info_impl_clearers() {
        clear_file_info();
    }
}

macro_rules! info_provider_iface {
    ($iface_init_fn:ident, $update_file_info_fn:ident, $cancel_update_fn:ident, $rust_provider:ident, $set_rust_provider:ident, $clear_rust_provider:ident) => {
        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_info_provider()` instead.
        unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusInfoProviderIface;
            unsafe {
                (*iface_struct).update_file_info = Some($update_file_info_fn);
                (*iface_struct).cancel_update = Some($cancel_update_fn);
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $update_file_info_fn(
            provider: *mut NautilusInfoProvider,
            file: *mut NautilusFileInfo,
            update_complete: *mut GClosure,
            handle: *mut *mut NautilusOperationHandle,
        ) -> NautilusOperationResult {
            let rust_provider = $rust_provider
                .lock()
                .ok()
                .and_then(|provider| provider.clone());
            let rust_provider = match rust_provider {
                Some(provider) => provider,
                None => return OperationResult::Complete.into(),
            };

            let (operation, raw_handle, state) =
                match new_operation(provider, file, update_complete) {
                    Some(operation) => operation,
                    None => return OperationResult::Failed.into(),
                };

            if !register_in_flight(raw_handle, state.clone()) {
                cleanup_state(&state);
                drop_raw_operation_handle(raw_handle);
                return OperationResult::Failed.into();
            }

            let result = catch_unwind(AssertUnwindSafe(|| {
                rust_provider.update_file_info_full(operation)
            }))
            .unwrap_or(OperationResult::Failed);

            if result == OperationResult::InProgress && !state.cancelled.load(Ordering::SeqCst) {
                if handle.is_null() {
                    finish_in_flight(raw_handle);
                    cleanup_state(&state);
                    return OperationResult::Failed.into();
                }

                unsafe {
                    *handle = raw_handle;
                }
                state.accepted.store(true, Ordering::SeqCst);
                OperationResult::InProgress.into()
            } else {
                finish_in_flight(raw_handle);
                cleanup_state(&state);
                result.into()
            }
        }

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        unsafe extern "C" fn $cancel_update_fn(
            _provider: *mut NautilusInfoProvider,
            handle: *mut NautilusOperationHandle,
        ) {
            let rust_provider = $rust_provider
                .lock()
                .ok()
                .and_then(|provider| provider.clone());
            let operation_handle = lookup_handle(handle);

            if let Some(state) = operation_handle.state.as_ref() {
                state.cancelled.store(true, Ordering::SeqCst);
            }

            if let Some(provider) = rust_provider {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    provider.cancel_update(&operation_handle);
                }));
            }

            if let Some(state) = operation_handle.state.as_ref() {
                finish_in_flight(handle);
                cleanup_state(state);
            }
        }

        fn $set_rust_provider(info_provider: Box<dyn InfoProvider>) {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = Some(Arc::from(info_provider));
            }
        }

        fn $clear_rust_provider() {
            if let Ok(mut provider) = $rust_provider.lock() {
                *provider = None;
            }
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Arc<dyn InfoProvider>>> = Mutex::new(None);
        }
    };
}

#[doc(hidden)]
pub const MAX_INFO_PROVIDERS: usize = 10;

#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_0, info_provider_update_file_info_0, info_provider_cancel_update_0, INFO_PROVIDER_0, set_info_provider_0, clear_info_provider_0);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_1, info_provider_update_file_info_1, info_provider_cancel_update_1, INFO_PROVIDER_1, set_info_provider_1, clear_info_provider_1);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_2, info_provider_update_file_info_2, info_provider_cancel_update_2, INFO_PROVIDER_2, set_info_provider_2, clear_info_provider_2);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_3, info_provider_update_file_info_3, info_provider_cancel_update_3, INFO_PROVIDER_3, set_info_provider_3, clear_info_provider_3);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_4, info_provider_update_file_info_4, info_provider_cancel_update_4, INFO_PROVIDER_4, set_info_provider_4, clear_info_provider_4);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_5, info_provider_update_file_info_5, info_provider_cancel_update_5, INFO_PROVIDER_5, set_info_provider_5, clear_info_provider_5);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_6, info_provider_update_file_info_6, info_provider_cancel_update_6, INFO_PROVIDER_6, set_info_provider_6, clear_info_provider_6);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_7, info_provider_update_file_info_7, info_provider_cancel_update_7, INFO_PROVIDER_7, set_info_provider_7, clear_info_provider_7);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_8, info_provider_update_file_info_8, info_provider_cancel_update_8, INFO_PROVIDER_8, set_info_provider_8, clear_info_provider_8);
#[rustfmt::skip] info_provider_iface!(info_provider_iface_init_9, info_provider_update_file_info_9, info_provider_cancel_update_9, INFO_PROVIDER_9, set_info_provider_9, clear_info_provider_9);

#[doc(hidden)]
pub fn info_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        info_provider_iface_init_0,
        info_provider_iface_init_1,
        info_provider_iface_init_2,
        info_provider_iface_init_3,
        info_provider_iface_init_4,
        info_provider_iface_init_5,
        info_provider_iface_init_6,
        info_provider_iface_init_7,
        info_provider_iface_init_8,
        info_provider_iface_init_9,
    ]
}

#[doc(hidden)]
pub fn rust_info_provider_setters() -> Vec<fn(Box<dyn InfoProvider>)> {
    vec![
        set_info_provider_0,
        set_info_provider_1,
        set_info_provider_2,
        set_info_provider_3,
        set_info_provider_4,
        set_info_provider_5,
        set_info_provider_6,
        set_info_provider_7,
        set_info_provider_8,
        set_info_provider_9,
    ]
}

fn rust_info_provider_clearers() -> Vec<fn()> {
    vec![
        clear_info_provider_0,
        clear_info_provider_1,
        clear_info_provider_2,
        clear_info_provider_3,
        clear_info_provider_4,
        clear_info_provider_5,
        clear_info_provider_6,
        clear_info_provider_7,
        clear_info_provider_8,
        clear_info_provider_9,
    ]
}

static RESERVED_INFO_PROVIDER_IFACE_SLOTS: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
pub fn take_next_info_provider_iface_index() -> Option<usize> {
    take_next_slot(&RESERVED_INFO_PROVIDER_IFACE_SLOTS, MAX_INFO_PROVIDERS)
}

pub(crate) fn release_info_provider_iface_index(index: usize) {
    if let Some(clear_provider) = rust_info_provider_clearers().get(index) {
        clear_provider();
    }

    release_slot(&RESERVED_INFO_PROVIDER_IFACE_SLOTS, index);
}

#[cfg(test)]
pub(crate) fn info_provider_slot_is_set(index: usize) -> bool {
    match index {
        0 => INFO_PROVIDER_0
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        1 => INFO_PROVIDER_1
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        2 => INFO_PROVIDER_2
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        3 => INFO_PROVIDER_3
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        4 => INFO_PROVIDER_4
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        5 => INFO_PROVIDER_5
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        6 => INFO_PROVIDER_6
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        7 => INFO_PROVIDER_7
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        8 => INFO_PROVIDER_8
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        9 => INFO_PROVIDER_9
            .lock()
            .map(|provider| provider.is_some())
            .unwrap_or(false),
        _ => false,
    }
}

#[doc(hidden)]
pub fn reset_info_provider_state() {
    reset_slots(&RESERVED_INFO_PROVIDER_IFACE_SLOTS);

    for clear_provider in rust_info_provider_clearers() {
        clear_provider();
    }

    let operations: Vec<(usize, Arc<OperationState>)> = match IN_FLIGHT_OPERATIONS.lock() {
        Ok(mut operations) => operations.drain().collect(),
        Err(_) => Vec::new(),
    };

    for (raw_handle, state) in operations {
        state.cancelled.store(true, Ordering::SeqCst);
        cleanup_state(&state);
        drop_raw_operation_handle(raw_handle as *mut NautilusOperationHandle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static COMPLETION_DESTROY_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
    static COMPLETION_ALREADY_DONE_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
    static COMPLETION_SCHEDULE_FAILURE_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
    static CANCEL_UPDATE_CALLS: AtomicUsize = AtomicUsize::new(0);
    #[cfg(not(nautilus_extension_rs_skip_link))]
    static ASYNC_UPDATE_CALLS: AtomicUsize = AtomicUsize::new(0);
    #[cfg(not(nautilus_extension_rs_skip_link))]
    static ASYNC_CANCEL_CALLS: AtomicUsize = AtomicUsize::new(0);

    struct PanickingCompletionDropPayload;

    impl Drop for PanickingCompletionDropPayload {
        fn drop(&mut self) {
            COMPLETION_DESTROY_DROP_CALLS.fetch_add(1, Ordering::SeqCst);
            panic!("completion source payload drop panic");
        }
    }

    struct CompletionScheduleFailurePayload;

    impl Drop for CompletionScheduleFailurePayload {
        fn drop(&mut self) {
            COMPLETION_SCHEDULE_FAILURE_DROP_CALLS.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct CompletionAlreadyDonePayload;

    impl Drop for CompletionAlreadyDonePayload {
        fn drop(&mut self) {
            COMPLETION_ALREADY_DONE_DROP_CALLS.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct PanickingCancelInfoProvider;

    impl InfoProvider for PanickingCancelInfoProvider {
        fn cancel_update(&self, handle: &OperationHandle) {
            assert_eq!(handle.id(), 0x1234);
            assert_eq!(handle.raw() as usize, 0x1234);
            assert!(!handle.is_cancelled());
            CANCEL_UPDATE_CALLS.fetch_add(1, Ordering::SeqCst);
            panic!("info provider cancel callback panic");
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    struct PanickingUpdateInfoProvider;

    #[cfg(not(nautilus_extension_rs_skip_link))]
    impl InfoProvider for PanickingUpdateInfoProvider {
        fn update_file_info_full(&self, _operation: UpdateFileInfoOperation) -> OperationResult {
            panic!("info provider update callback panic");
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    struct AsyncCancelInfoProvider;

    #[cfg(not(nautilus_extension_rs_skip_link))]
    impl InfoProvider for AsyncCancelInfoProvider {
        fn update_file_info_full(&self, operation: UpdateFileInfoOperation) -> OperationResult {
            let pending = operation.into_pending();
            assert!(!pending.handle().is_cancelled());
            ASYNC_UPDATE_CALLS.fetch_add(1, Ordering::SeqCst);
            OperationResult::InProgress
        }

        fn cancel_update(&self, handle: &OperationHandle) {
            assert!(handle.is_cancelled());
            ASYNC_CANCEL_CALLS.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    unsafe extern "C" fn noop_update_complete() {}

    #[cfg(not(nautilus_extension_rs_skip_link))]
    fn test_update_complete_closure() -> *mut GClosure {
        unsafe {
            crate::gobject_ffi::g_cclosure_new(Some(noop_update_complete), ptr::null_mut(), None)
        }
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    fn opaque_native_file_info() -> FileInfo {
        let raw = unsafe {
            crate::gobject_ffi::g_object_new(
                crate::gobject_ffi::G_TYPE_OBJECT,
                ptr::null::<c_char>(),
            )
        };

        unsafe { FileInfo::from_raw_full(raw as *mut NautilusFileInfo) }
            .expect("GObject should be constructible for opaque native FileInfo tests")
    }

    fn test_operation_state(main_context: usize) -> Arc<OperationState> {
        Arc::new(OperationState {
            provider: 0,
            file_info: 0,
            closure: 0,
            main_context,
            raw_handle: AtomicUsize::new(0),
            accepted: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            completed: AtomicBool::new(false),
            cleaned: AtomicBool::new(false),
            cleanup_lock: Mutex::new(()),
        })
    }

    #[test]
    fn operation_result_round_trips_through_sys_enum() {
        let cases = [
            (
                OperationResult::Complete,
                NautilusOperationResult::NautilusOperationComplete,
            ),
            (
                OperationResult::Failed,
                NautilusOperationResult::NautilusOperationFailed,
            ),
            (
                OperationResult::InProgress,
                NautilusOperationResult::NautilusOperationInProgress,
            ),
        ];

        for (safe, raw) in cases {
            assert_eq!(NautilusOperationResult::from(safe), raw);
            assert_eq!(OperationResult::from(raw), safe);
        }
    }

    #[test]
    fn unknown_operation_handle_is_not_cancelled() {
        let handle = OperationHandle::unknown(0x1234usize as *mut NautilusOperationHandle);

        assert_eq!(handle.id(), 0x1234);
        assert_eq!(handle.raw() as usize, 0x1234);
        assert!(!handle.is_cancelled());
    }

    #[test]
    fn raw_update_handle_is_exposed_only_for_in_progress_results() {
        let raw = 0x1234usize as *mut NautilusOperationHandle;

        assert!(operation_handle_for_update_result(OperationResult::Complete, raw).is_none());
        assert!(operation_handle_for_update_result(OperationResult::Failed, raw).is_none());
        assert!(
            operation_handle_for_update_result(OperationResult::InProgress, ptr::null_mut())
                .is_none()
        );

        let handle = operation_handle_for_update_result(OperationResult::InProgress, raw).unwrap();
        assert_eq!(handle.raw(), raw);
    }

    #[test]
    fn in_flight_registry_releases_map_and_raw_handle_refs() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_info_provider_state();

        let state = test_operation_state(0);
        let raw = Arc::into_raw(state.clone()) as *mut NautilusOperationHandle;
        state.raw_handle.store(raw as usize, Ordering::SeqCst);

        assert_eq!(Arc::strong_count(&state), 2);
        assert!(register_in_flight(raw, state.clone()));
        assert_eq!(Arc::strong_count(&state), 3);

        assert!(finish_in_flight(raw));
        assert_eq!(Arc::strong_count(&state), 1);
        assert!(!finish_in_flight(raw));
    }

    #[test]
    fn full_transfer_file_info_list_rejects_null() {
        assert!(unsafe { FileInfo::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { FileInfo::from_raw_borrowed(ptr::null_mut()) }.is_none());
        assert!(unsafe { FileInfoList::from_raw_full(ptr::null_mut()) }.is_none());
    }

    #[test]
    fn borrowed_file_info_handle_rejects_null_and_preserves_raw_pointer() {
        assert!(unsafe { FileInfoHandle::from_raw(ptr::null_mut()) }.is_none());

        let raw = 0x1234usize as *mut NautilusFileInfo;
        let handle = unsafe { FileInfoHandle::from_raw(raw) }.unwrap();

        assert_eq!(handle.raw(), raw);
        assert_eq!(handle.as_ptr(), raw);
    }

    #[test]
    fn generic_owned_gobject_rejects_null() {
        assert!(unsafe { OwnedGObject::<GObject>::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { OwnedGObject::<GObject>::from_raw_borrowed(ptr::null_mut()) }.is_none());
    }

    #[test]
    fn info_provider_handle_rejects_null_raw_pointers() {
        assert!(unsafe { InfoProviderHandle::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { InfoProviderHandle::from_raw_borrowed(ptr::null_mut()) }.is_none());
    }

    #[test]
    fn update_complete_callback_rejects_null_and_preserves_full_transfer_raw_pointer() {
        assert!(unsafe { UpdateCompleteCallback::from_raw_full(ptr::null_mut()) }.is_none());
        assert!(unsafe { UpdateCompleteCallback::from_raw_borrowed(ptr::null_mut()) }.is_none());

        let raw = 0x1234usize as *mut GClosure;
        let callback = unsafe { UpdateCompleteCallback::from_raw_full(raw) }.unwrap();

        assert_eq!(callback.raw(), raw);
        assert_eq!(callback.as_ptr(), raw);
        assert_eq!(callback.into_raw(), raw);
    }

    #[test]
    fn no_link_update_complete_callback_invoke_is_inert() {
        let mut callback = UpdateCompleteCallback {
            raw: 0x1234usize as *mut GClosure,
        };
        let provider = InfoProviderHandle {
            raw: ptr::null_mut(),
        };
        let handle = OperationHandle::unknown(ptr::null_mut());

        assert!(!unsafe { callback.invoke(&provider, &handle, OperationResult::Complete) });

        callback.raw = ptr::null_mut();
    }

    #[test]
    fn info_provider_iface_update_completes_when_no_provider_is_registered() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_info_provider_state();

        let mut iface: NautilusInfoProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            info_provider_iface_init_0(
                &mut iface as *mut NautilusInfoProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let mut handle: *mut NautilusOperationHandle = ptr::null_mut();
        let result = unsafe {
            (iface.update_file_info.unwrap())(
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut handle,
            )
        };

        assert_eq!(OperationResult::from(result), OperationResult::Complete);
        assert!(handle.is_null());

        reset_info_provider_state();
    }

    #[test]
    fn info_provider_iface_update_fails_when_update_callback_is_missing() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_info_provider_state();

        set_info_provider_0(Box::new(PanickingCancelInfoProvider));

        let mut iface: NautilusInfoProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            info_provider_iface_init_0(
                &mut iface as *mut NautilusInfoProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let mut handle: *mut NautilusOperationHandle = ptr::null_mut();
        let result = unsafe {
            (iface.update_file_info.unwrap())(
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut handle,
            )
        };

        assert_eq!(OperationResult::from(result), OperationResult::Failed);
        assert!(handle.is_null());

        reset_info_provider_state();
    }

    #[test]
    fn info_provider_iface_cancel_routes_unknown_handles_and_catches_provider_panics() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_info_provider_state();
        CANCEL_UPDATE_CALLS.store(0, Ordering::SeqCst);

        set_info_provider_0(Box::new(PanickingCancelInfoProvider));

        let mut iface: NautilusInfoProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            info_provider_iface_init_0(
                &mut iface as *mut NautilusInfoProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let result = std::panic::catch_unwind(|| unsafe {
            (iface.cancel_update.unwrap())(
                ptr::null_mut(),
                0x1234usize as *mut NautilusOperationHandle,
            );
        });

        assert!(result.is_ok());
        assert_eq!(CANCEL_UPDATE_CALLS.load(Ordering::SeqCst), 1);

        reset_info_provider_state();
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn info_provider_iface_update_catches_provider_panics_with_real_native_inputs() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_info_provider_state();

        set_info_provider_0(Box::new(PanickingUpdateInfoProvider));

        let file = opaque_native_file_info();
        let closure = test_update_complete_closure();
        assert!(!closure.is_null());

        let mut iface: NautilusInfoProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            info_provider_iface_init_0(
                &mut iface as *mut NautilusInfoProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        let mut handle: *mut NautilusOperationHandle = ptr::null_mut();
        let result = unsafe {
            (iface.update_file_info.unwrap())(ptr::null_mut(), file.raw(), closure, &mut handle)
        };

        assert_eq!(OperationResult::from(result), OperationResult::Failed);
        assert!(handle.is_null());

        unsafe {
            crate::gobject_ffi::g_closure_unref(closure);
        }

        reset_info_provider_state();
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn info_provider_async_cancel_stress_uses_real_native_handles() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_info_provider_state();
        ASYNC_UPDATE_CALLS.store(0, Ordering::SeqCst);
        ASYNC_CANCEL_CALLS.store(0, Ordering::SeqCst);

        set_info_provider_0(Box::new(AsyncCancelInfoProvider));

        let mut iface: NautilusInfoProviderIface = unsafe { std::mem::zeroed() };
        unsafe {
            info_provider_iface_init_0(
                &mut iface as *mut NautilusInfoProviderIface as gpointer,
                ptr::null_mut(),
            );
        }

        for _ in 0..64 {
            let file = opaque_native_file_info();
            let closure = test_update_complete_closure();
            assert!(!closure.is_null());

            let mut handle: *mut NautilusOperationHandle = ptr::null_mut();
            let result = unsafe {
                (iface.update_file_info.unwrap())(ptr::null_mut(), file.raw(), closure, &mut handle)
            };

            assert_eq!(OperationResult::from(result), OperationResult::InProgress);
            assert!(!handle.is_null());

            unsafe {
                (iface.cancel_update.unwrap())(ptr::null_mut(), handle);
                crate::gobject_ffi::g_closure_unref(closure);
            }
        }

        assert_eq!(ASYNC_UPDATE_CALLS.load(Ordering::SeqCst), 64);
        assert_eq!(ASYNC_CANCEL_CALLS.load(Ordering::SeqCst), 64);

        reset_info_provider_state();
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn documented_type_accessors_are_inert_in_no_link_mode() {
        assert_eq!(OperationResult::type_(), 0);
        assert_eq!(FileInfo::type_(), 0);
        assert_eq!(FileInfoHandle::type_(), 0);
        assert_eq!(InfoProviderHandle::type_(), 0);
    }

    #[cfg(nautilus_extension_rs_skip_link)]
    #[test]
    fn file_info_native_helpers_are_inert_in_no_link_mode() {
        assert!(FileInfo::create_for_uri("file:///tmp/example").is_none());
        assert!(FileInfo::lookup_for_uri("file:///tmp/example").is_none());
        assert!(unsafe { FileInfoList::copy_from_raw(0x5678usize as *mut GList) }.is_none());
        assert!(FileInfoList::copy(&[]).is_none());

        let create_location =
            unsafe { OwnedGObject::<GFile>::from_raw_full(0x5678usize as *mut GFile) }.unwrap();
        assert!(FileInfo::create(&create_location).is_none());
        assert_eq!(create_location.into_raw() as usize, 0x5678);

        let lookup_location =
            unsafe { OwnedGObject::<GFile>::from_raw_full(0x6789usize as *mut GFile) }.unwrap();
        assert!(FileInfo::lookup(&lookup_location).is_none());
        assert_eq!(lookup_location.into_raw() as usize, 0x6789);

        let file_info = FileInfo {
            raw_file_info: 0x1234usize as *mut NautilusFileInfo,
        };

        assert!(!file_info.is_gone());
        assert_eq!(file_info.name(), None);
        assert_eq!(file_info.uri(), None);
        assert_eq!(file_info.parent_uri(), None);
        assert_eq!(file_info.uri_scheme(), None);
        assert_eq!(file_info.mime_type(), None);
        assert!(!file_info.is_mime_type("text/plain"));
        assert!(!file_info.is_directory());
        file_info.add_emblem("important");
        assert_eq!(file_info.string_attribute("example"), None);
        file_info.add_string_attribute("example", "value");
        file_info.invalidate_extension_info();
        assert_eq!(file_info.activation_uri(), None);
        assert_eq!(file_info.file_type(), G_FILE_TYPE_UNKNOWN);
        assert!(file_info.location().is_none());
        assert!(file_info.location_uri().is_none());
        assert!(file_info.location_path().is_none());
        assert!(file_info.parent_location().is_none());
        assert!(file_info.parent_info().is_none());
        assert!(file_info.mount().is_none());
        assert!(!file_info.can_write());
        assert_eq!(file_info.get_uri(), "");
        assert_eq!(file_info.get_uri_scheme(), "");

        assert_eq!(file_info.into_raw() as usize, 0x1234);
    }

    #[cfg(not(nautilus_extension_rs_skip_link))]
    #[test]
    fn documented_type_accessors_return_registered_gtypes() {
        assert_ne!(OperationResult::type_(), 0);
        assert_ne!(FileInfo::type_(), 0);
        assert_ne!(FileInfoHandle::type_(), 0);
        assert_ne!(InfoProviderHandle::type_(), 0);
    }

    #[test]
    fn completion_source_destroy_callback_catches_panicking_payload_drop() {
        COMPLETION_DESTROY_DROP_CALLS.store(0, Ordering::SeqCst);

        let state = test_operation_state(0);
        let payload = PanickingCompletionDropPayload;
        let source = Box::new(CompletionSource {
            state,
            result: OperationResult::Complete,
            update_file_info: Some(Box::new(move |_| {
                let _ = &payload;
            })),
        });

        unsafe {
            destroy_completion_source(Box::into_raw(source) as gpointer);
        }

        assert_eq!(COMPLETION_DESTROY_DROP_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn failed_completion_schedule_does_not_consume_completion() {
        COMPLETION_SCHEDULE_FAILURE_DROP_CALLS.store(0, Ordering::SeqCst);

        let state = test_operation_state(0);
        let completion = UpdateCompletion {
            state: state.clone(),
        };
        let payload = CompletionScheduleFailurePayload;

        assert!(
            !completion.complete_with(OperationResult::Complete, move |_| {
                let _ = &payload;
            })
        );
        assert!(!state.completed.load(Ordering::SeqCst));
        assert_eq!(
            COMPLETION_SCHEDULE_FAILURE_DROP_CALLS.load(Ordering::SeqCst),
            1
        );

        assert!(!completion.complete(OperationResult::Failed));
        assert!(!state.completed.load(Ordering::SeqCst));
    }

    #[test]
    fn already_completed_completion_rejects_follow_up_and_drops_payload() {
        COMPLETION_ALREADY_DONE_DROP_CALLS.store(0, Ordering::SeqCst);

        let state = test_operation_state(0);
        state.completed.store(true, Ordering::SeqCst);
        let completion = UpdateCompletion {
            state: state.clone(),
        };
        let payload = CompletionAlreadyDonePayload;

        assert!(
            !completion.complete_with(OperationResult::Complete, move |_| {
                let _ = &payload;
            })
        );
        assert!(state.completed.load(Ordering::SeqCst));
        assert_eq!(COMPLETION_ALREADY_DONE_DROP_CALLS.load(Ordering::SeqCst), 1);
    }

    struct DefaultFileInfoImpl;

    impl FileInfoImpl for DefaultFileInfoImpl {}

    #[test]
    fn file_info_impl_defaults_are_empty_and_safe() {
        let file_info = DefaultFileInfoImpl;
        let handle = FileInfoHandle {
            raw_file_info: 0x1234usize as *mut NautilusFileInfo,
        };

        assert!(!file_info.is_gone(handle));
        assert_eq!(file_info.name(handle), None);
        assert_eq!(file_info.uri(handle), None);
        assert_eq!(file_info.parent_uri(handle), None);
        assert_eq!(file_info.uri_scheme(handle), None);
        assert_eq!(file_info.mime_type(handle), None);
        assert!(!file_info.is_mime_type(handle, "text/plain"));
        assert!(!file_info.is_directory(handle));
        assert_eq!(file_info.string_attribute(handle, "example"), None);
        assert_eq!(file_info.activation_uri(handle), None);
        assert_eq!(file_info.file_type(handle), G_FILE_TYPE_UNKNOWN);
        assert!(file_info.location(handle).is_none());
        assert!(file_info.parent_location(handle).is_none());
        assert!(file_info.parent_info(handle).is_none());
        assert!(file_info.mount(handle).is_none());
        assert!(!file_info.can_write(handle));
    }

    struct MimeFileInfoImpl;

    impl FileInfoImpl for MimeFileInfoImpl {
        fn mime_type(&self, _file_info: FileInfoHandle) -> Option<String> {
            Some("text/plain".to_string())
        }
    }

    struct PanickingFileInfoImpl;

    impl FileInfoImpl for PanickingFileInfoImpl {
        fn is_gone(&self, _file_info: FileInfoHandle) -> bool {
            panic!("file-info is_gone panic");
        }

        fn name(&self, _file_info: FileInfoHandle) -> Option<String> {
            panic!("file-info name panic");
        }

        fn uri(&self, _file_info: FileInfoHandle) -> Option<String> {
            panic!("file-info uri panic");
        }

        fn parent_uri(&self, _file_info: FileInfoHandle) -> Option<String> {
            panic!("file-info parent_uri panic");
        }

        fn uri_scheme(&self, _file_info: FileInfoHandle) -> Option<String> {
            panic!("file-info uri_scheme panic");
        }

        fn mime_type(&self, _file_info: FileInfoHandle) -> Option<String> {
            panic!("file-info mime_type panic");
        }

        fn is_mime_type(&self, _file_info: FileInfoHandle, _mime_type: &str) -> bool {
            panic!("file-info is_mime_type panic");
        }

        fn is_directory(&self, _file_info: FileInfoHandle) -> bool {
            panic!("file-info is_directory panic");
        }

        fn add_emblem(&self, _file_info: FileInfoHandle, _emblem_name: &str) {
            panic!("file-info add_emblem panic");
        }

        fn string_attribute(
            &self,
            _file_info: FileInfoHandle,
            _attribute_name: &str,
        ) -> Option<String> {
            panic!("file-info string_attribute panic");
        }

        fn add_string_attribute(
            &self,
            _file_info: FileInfoHandle,
            _attribute_name: &str,
            _value: &str,
        ) {
            panic!("file-info add_string_attribute panic");
        }

        fn invalidate_extension_info(&self, _file_info: FileInfoHandle) {
            panic!("file-info invalidate_extension_info panic");
        }

        fn activation_uri(&self, _file_info: FileInfoHandle) -> Option<String> {
            panic!("file-info activation_uri panic");
        }

        fn file_type(&self, _file_info: FileInfoHandle) -> GFileType {
            panic!("file-info file_type panic");
        }

        fn location(&self, _file_info: FileInfoHandle) -> Option<OwnedGObject<GFile>> {
            panic!("file-info location panic");
        }

        fn parent_location(&self, _file_info: FileInfoHandle) -> Option<OwnedGObject<GFile>> {
            panic!("file-info parent_location panic");
        }

        fn parent_info(&self, _file_info: FileInfoHandle) -> Option<FileInfo> {
            panic!("file-info parent_info panic");
        }

        fn mount(&self, _file_info: FileInfoHandle) -> Option<OwnedGObject<GMount>> {
            panic!("file-info mount panic");
        }

        fn can_write(&self, _file_info: FileInfoHandle) -> bool {
            panic!("file-info can_write panic");
        }
    }

    #[test]
    fn file_info_iface_trampolines_catch_all_provider_panics() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_file_info_impl_state();

        set_file_info_impl_0(Box::new(PanickingFileInfoImpl));

        let mut iface: NautilusFileInfoInterface = unsafe { std::mem::zeroed() };
        unsafe {
            file_info_iface_init_0(
                &mut iface as *mut NautilusFileInfoInterface as gpointer,
                ptr::null_mut(),
            );
        }

        let raw_file_info = 0x1234usize as *mut NautilusFileInfo;
        let mime_type = CString::new("text/plain").unwrap();
        let emblem_name = CString::new("important").unwrap();
        let attribute_name = CString::new("example").unwrap();
        let value = CString::new("value").unwrap();

        assert_eq!(unsafe { (iface.is_gone.unwrap())(raw_file_info) }, GFALSE);
        assert!(unsafe { (iface.get_name.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_uri.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_parent_uri.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_uri_scheme.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_mime_type.unwrap())(raw_file_info) }.is_null());
        assert_eq!(
            unsafe { (iface.is_mime_type.unwrap())(raw_file_info, mime_type.as_ptr()) },
            GFALSE
        );
        assert_eq!(
            unsafe { (iface.is_directory.unwrap())(raw_file_info) },
            GFALSE
        );
        assert!(unsafe {
            (iface.get_string_attribute.unwrap())(raw_file_info, attribute_name.as_ptr())
        }
        .is_null());
        assert!(unsafe { (iface.get_activation_uri.unwrap())(raw_file_info) }.is_null());
        assert_eq!(
            unsafe { (iface.get_file_type.unwrap())(raw_file_info) },
            G_FILE_TYPE_UNKNOWN
        );
        assert!(unsafe { (iface.get_location.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_parent_location.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_parent_info.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_mount.unwrap())(raw_file_info) }.is_null());
        assert_eq!(unsafe { (iface.can_write.unwrap())(raw_file_info) }, GFALSE);

        unsafe {
            (iface.add_emblem.unwrap())(raw_file_info, emblem_name.as_ptr());
            (iface.add_string_attribute.unwrap())(
                raw_file_info,
                attribute_name.as_ptr(),
                value.as_ptr(),
            );
            (iface.invalidate_extension_info.unwrap())(raw_file_info);
        }

        reset_file_info_impl_state();
    }

    #[test]
    fn file_info_impl_default_mime_match_uses_mime_type() {
        let file_info = MimeFileInfoImpl;
        let handle = FileInfoHandle {
            raw_file_info: 0x1234usize as *mut NautilusFileInfo,
        };

        assert!(file_info.is_mime_type(handle, "text/plain"));
        assert!(!file_info.is_mime_type(handle, "image/png"));
    }

    static ADD_EMBLEM_CALLS: AtomicUsize = AtomicUsize::new(0);
    static ADD_STRING_ATTRIBUTE_CALLS: AtomicUsize = AtomicUsize::new(0);
    static INVALIDATE_EXTENSION_INFO_CALLS: AtomicUsize = AtomicUsize::new(0);

    struct RoutedFileInfoImpl;

    impl FileInfoImpl for RoutedFileInfoImpl {
        fn is_gone(&self, file_info: FileInfoHandle) -> bool {
            file_info.raw() as usize == 0x1234
        }

        fn name(&self, _file_info: FileInfoHandle) -> Option<String> {
            Some("demo-name".to_string())
        }

        fn uri(&self, _file_info: FileInfoHandle) -> Option<String> {
            Some("file:///tmp/demo-name".to_string())
        }

        fn parent_uri(&self, _file_info: FileInfoHandle) -> Option<String> {
            Some("file:///tmp".to_string())
        }

        fn uri_scheme(&self, _file_info: FileInfoHandle) -> Option<String> {
            Some("file".to_string())
        }

        fn mime_type(&self, _file_info: FileInfoHandle) -> Option<String> {
            Some("text/plain".to_string())
        }

        fn is_mime_type(&self, _file_info: FileInfoHandle, mime_type: &str) -> bool {
            mime_type == "text/plain"
        }

        fn is_directory(&self, _file_info: FileInfoHandle) -> bool {
            true
        }

        fn add_emblem(&self, _file_info: FileInfoHandle, emblem_name: &str) {
            if emblem_name == "important" {
                ADD_EMBLEM_CALLS.fetch_add(1, Ordering::SeqCst);
            }
        }

        fn string_attribute(
            &self,
            _file_info: FileInfoHandle,
            attribute_name: &str,
        ) -> Option<String> {
            Some(format!("attribute:{attribute_name}"))
        }

        fn add_string_attribute(
            &self,
            _file_info: FileInfoHandle,
            attribute_name: &str,
            value: &str,
        ) {
            if attribute_name == "example" && value == "value" {
                ADD_STRING_ATTRIBUTE_CALLS.fetch_add(1, Ordering::SeqCst);
            }
        }

        fn invalidate_extension_info(&self, _file_info: FileInfoHandle) {
            INVALIDATE_EXTENSION_INFO_CALLS.fetch_add(1, Ordering::SeqCst);
        }

        fn activation_uri(&self, _file_info: FileInfoHandle) -> Option<String> {
            Some("file:///tmp/demo-name".to_string())
        }

        fn file_type(&self, _file_info: FileInfoHandle) -> GFileType {
            crate::gio_ffi::G_FILE_TYPE_REGULAR
        }

        fn can_write(&self, _file_info: FileInfoHandle) -> bool {
            true
        }
    }

    #[test]
    fn file_info_iface_trampolines_route_to_registered_impl() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_file_info_impl_state();
        ADD_EMBLEM_CALLS.store(0, Ordering::SeqCst);
        ADD_STRING_ATTRIBUTE_CALLS.store(0, Ordering::SeqCst);
        INVALIDATE_EXTENSION_INFO_CALLS.store(0, Ordering::SeqCst);

        set_file_info_impl_0(Box::new(RoutedFileInfoImpl));

        let mut iface: NautilusFileInfoInterface = unsafe { std::mem::zeroed() };
        unsafe {
            file_info_iface_init_0(
                &mut iface as *mut NautilusFileInfoInterface as gpointer,
                ptr::null_mut(),
            );
        }

        let raw_file_info = 0x1234usize as *mut NautilusFileInfo;
        let mime_type = CString::new("text/plain").unwrap();
        let emblem_name = CString::new("important").unwrap();
        let attribute_name = CString::new("example").unwrap();
        let value = CString::new("value").unwrap();

        assert_eq!(unsafe { (iface.is_gone.unwrap())(raw_file_info) }, GTRUE);
        assert_eq!(
            unsafe { take_glib_string((iface.get_name.unwrap())(raw_file_info)) },
            Some("demo-name".to_string())
        );
        assert_eq!(
            unsafe { take_glib_string((iface.get_uri.unwrap())(raw_file_info)) },
            Some("file:///tmp/demo-name".to_string())
        );
        assert_eq!(
            unsafe { take_glib_string((iface.get_parent_uri.unwrap())(raw_file_info)) },
            Some("file:///tmp".to_string())
        );
        assert_eq!(
            unsafe { take_glib_string((iface.get_uri_scheme.unwrap())(raw_file_info)) },
            Some("file".to_string())
        );
        assert_eq!(
            unsafe { take_glib_string((iface.get_mime_type.unwrap())(raw_file_info)) },
            Some("text/plain".to_string())
        );
        assert_eq!(
            unsafe { (iface.is_mime_type.unwrap())(raw_file_info, mime_type.as_ptr()) },
            GTRUE
        );
        assert_eq!(
            unsafe { (iface.is_directory.unwrap())(raw_file_info) },
            GTRUE
        );
        assert_eq!(
            unsafe {
                take_glib_string((iface.get_string_attribute.unwrap())(
                    raw_file_info,
                    attribute_name.as_ptr(),
                ))
            },
            Some("attribute:example".to_string())
        );
        assert_eq!(
            unsafe { take_glib_string((iface.get_activation_uri.unwrap())(raw_file_info)) },
            Some("file:///tmp/demo-name".to_string())
        );
        assert_eq!(
            unsafe { (iface.get_file_type.unwrap())(raw_file_info) },
            crate::gio_ffi::G_FILE_TYPE_REGULAR
        );
        assert!(unsafe { (iface.get_location.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_parent_location.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_parent_info.unwrap())(raw_file_info) }.is_null());
        assert!(unsafe { (iface.get_mount.unwrap())(raw_file_info) }.is_null());
        assert_eq!(unsafe { (iface.can_write.unwrap())(raw_file_info) }, GTRUE);

        unsafe {
            (iface.add_emblem.unwrap())(raw_file_info, emblem_name.as_ptr());
            (iface.add_string_attribute.unwrap())(
                raw_file_info,
                attribute_name.as_ptr(),
                value.as_ptr(),
            );
            (iface.invalidate_extension_info.unwrap())(raw_file_info);
        }

        assert_eq!(ADD_EMBLEM_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(ADD_STRING_ATTRIBUTE_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(INVALIDATE_EXTENSION_INFO_CALLS.load(Ordering::SeqCst), 1);

        reset_file_info_impl_state();
    }

    #[test]
    fn reset_file_info_impl_state_allows_slot_reuse() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_file_info_impl_state();

        for expected_index in 0..MAX_FILE_INFO_IMPLS {
            assert_eq!(take_next_file_info_iface_index(), Some(expected_index));
        }

        assert_eq!(take_next_file_info_iface_index(), None);

        reset_file_info_impl_state();

        assert_eq!(take_next_file_info_iface_index(), Some(0));

        reset_file_info_impl_state();
    }

    #[test]
    fn reset_info_provider_state_allows_slot_reuse() {
        let _guard = crate::test_support::PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");
        reset_info_provider_state();

        for expected_index in 0..MAX_INFO_PROVIDERS {
            assert_eq!(take_next_info_provider_iface_index(), Some(expected_index));
        }

        assert_eq!(take_next_info_provider_iface_index(), None);

        reset_info_provider_state();

        assert_eq!(take_next_info_provider_iface_index(), Some(0));

        reset_info_provider_state();
    }
}
