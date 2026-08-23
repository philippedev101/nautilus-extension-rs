use super::*;

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
    /// Returns the registered `NautilusOperationResult` GType.
    pub fn type_() -> GType {
        // SAFETY: the Nautilus GType registration functions take no arguments and are safe
        // to call at any point.
        unsafe { nautilus_operation_result_get_type() }
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
    pub(crate) raw: *mut NautilusInfoProvider,
}

impl InfoProviderHandle {
    /// Returns the registered `NautilusInfoProvider` GType.
    pub fn type_() -> GType {
        // SAFETY: the Nautilus GType registration functions take no arguments and are safe
        // to call at any point.
        unsafe { nautilus_info_provider_get_type() }
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

        // SAFETY: the caller guarantees the pointer is a live object, and null was rejected
        // just above. This takes the reference the returned wrapper owns.
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
        // SAFETY: `self.raw` is the live Nautilus object this wrapper owns, and the
        // arguments outlive the call.
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
        // SAFETY: the caller upholds the contract documented on `update_file_info_raw`.
        unsafe { self.update_file_info_raw(file, update_complete.raw()) }
    }

    /// Calls the provider's raw cancel function for an operation handle.
    pub fn cancel_update(&self, handle: &OperationHandle) {
        let raw = handle.raw();
        if raw.is_null() {
            return;
        }

        // SAFETY: `self.raw` is the live Nautilus object this wrapper owns.
        unsafe {
            nautilus_info_provider_cancel_update(self.raw, raw);
        }
    }
}

impl Clone for InfoProviderHandle {
    fn clone(&self) -> InfoProviderHandle {
        // SAFETY: the wrapper holds a live reference to this object, so taking one more is
        // sound.
        unsafe {
            g_object_ref(self.raw as *mut GObject);
        }

        InfoProviderHandle { raw: self.raw }
    }
}

impl Drop for InfoProviderHandle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: the wrapper owns the reference being released and does not use the
            // pointer again.
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
    pub(crate) raw: *mut GClosure,
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

        // SAFETY: the caller guarantees `raw` is a live closure, and null was rejected
        // just above. This takes the reference the returned wrapper owns.
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
        if !NATIVE_API_AVAILABLE
            || self.raw.is_null()
            || provider.raw().is_null()
            || handle.raw().is_null()
        {
            return false;
        }

        // SAFETY: `self.raw` is the live Nautilus object this wrapper owns, and the
        // arguments outlive the call.
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
}

impl Clone for UpdateCompleteCallback {
    fn clone(&self) -> UpdateCompleteCallback {
        // SAFETY: the wrapper holds a live reference to this closure.
        unsafe {
            g_closure_ref(self.raw);
        }

        UpdateCompleteCallback { raw: self.raw }
    }
}

impl Drop for UpdateCompleteCallback {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: the wrapper owns the closure reference being released.
            unsafe {
                g_closure_unref(self.raw);
            }
        }
    }
}
