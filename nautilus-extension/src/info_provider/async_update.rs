use super::*;

#[derive(Clone)]
/// Opaque handle used to identify an asynchronous InfoProvider operation.
pub struct OperationHandle {
    raw: usize,
    pub(crate) state: Option<Arc<OperationState>>,
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

    pub(crate) fn unknown(raw: *mut NautilusOperationHandle) -> OperationHandle {
        OperationHandle {
            raw: raw as usize,
            state: None,
        }
    }
}

/// Completion callback for an asynchronous InfoProvider operation.
pub struct UpdateCompletion {
    pub(crate) state: Arc<OperationState>,
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
    pub(crate) file_info: FileInfo,
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

pub(crate) struct OperationState {
    pub(crate) provider: usize,
    pub(crate) file_info: usize,
    pub(crate) closure: usize,
    pub(crate) main_context: usize,
    pub(crate) raw_handle: AtomicUsize,
    pub(crate) accepted: AtomicBool,
    pub(crate) cancelled: AtomicBool,
    pub(crate) completed: AtomicBool,
    pub(crate) cleaned: AtomicBool,
    pub(crate) cleanup_lock: Mutex<()>,
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

    fn raw_handle(&self) -> *mut NautilusOperationHandle {
        self.raw_handle.load(Ordering::SeqCst) as *mut NautilusOperationHandle
    }
}

type UpdateFileInfoFn = Box<dyn FnOnce(&mut FileInfo) + Send + 'static>;

pub(crate) struct CompletionSource {
    pub(crate) state: Arc<OperationState>,
    pub(crate) result: OperationResult,
    pub(crate) update_file_info: Option<UpdateFileInfoFn>,
}

pub(crate) unsafe extern "C" fn run_completion_source(data: gpointer) -> gboolean {
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

pub(crate) unsafe extern "C" fn destroy_completion_source(data: gpointer) {
    if !data.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(unsafe { Box::from_raw(data as *mut CompletionSource) });
        }));
    }
}

pub(crate) fn ref_main_context_for_source(state: &OperationState) -> Option<*mut GMainContext> {
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

pub(crate) fn schedule_completion_source(source_data: CompletionSource) -> bool {
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

lazy_static! {
    pub(crate) static ref IN_FLIGHT_OPERATIONS: Mutex<HashMap<usize, Arc<OperationState>>> =
        Mutex::new(HashMap::new());
}

pub(crate) fn register_in_flight(
    raw: *mut NautilusOperationHandle,
    state: Arc<OperationState>,
) -> bool {
    match IN_FLIGHT_OPERATIONS.lock() {
        Ok(mut operations) => {
            operations.insert(raw as usize, state);
            true
        }
        Err(_) => false,
    }
}

pub(crate) fn lookup_handle(raw: *mut NautilusOperationHandle) -> OperationHandle {
    let state = IN_FLIGHT_OPERATIONS
        .lock()
        .ok()
        .and_then(|operations| operations.get(&(raw as usize)).cloned());

    match state {
        Some(state) => OperationHandle::new(raw, state),
        None => OperationHandle::unknown(raw),
    }
}

pub(crate) fn finish_in_flight(raw: *mut NautilusOperationHandle) -> bool {
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

pub(crate) fn drop_raw_operation_handle(raw: *mut NautilusOperationHandle) {
    if !raw.is_null() {
        unsafe {
            drop(Arc::from_raw(raw as *const OperationState));
        }
    }
}

pub(crate) fn cleanup_state(state: &OperationState) {
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

pub(crate) fn new_operation(
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

pub(crate) fn bool_to_gboolean(value: bool) -> gboolean {
    if value {
        GTRUE
    } else {
        GFALSE
    }
}

pub(crate) fn operation_handle_for_update_result(
    result: OperationResult,
    raw_handle: *mut NautilusOperationHandle,
) -> Option<OperationHandle> {
    if result == OperationResult::InProgress && !raw_handle.is_null() {
        Some(OperationHandle::unknown(raw_handle))
    } else {
        None
    }
}

pub(crate) fn dup_optional_string(value: Option<String>) -> *mut c_char {
    let value = match value.and_then(|value| CString::new(value).ok()) {
        Some(value) => value,
        None => return ptr::null_mut(),
    };

    unsafe { g_strdup(value.as_ptr()) }
}

pub(crate) unsafe fn c_string_arg(raw: *const c_char) -> Option<String> {
    if raw.is_null() {
        return None;
    }

    Some(
        unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned(),
    )
}

pub(crate) fn owned_gobject_to_raw<T>(object: Option<OwnedGObject<T>>) -> *mut T {
    object
        .map(|object| object.into_raw())
        .unwrap_or(ptr::null_mut())
}

pub(crate) fn file_info_to_raw(file_info: Option<FileInfo>) -> *mut NautilusFileInfo {
    file_info
        .map(|file_info| file_info.into_raw())
        .unwrap_or(ptr::null_mut())
}

pub(crate) fn with_file_info_impl<R, F>(
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
