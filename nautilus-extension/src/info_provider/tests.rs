use super::*;
use super::{file_info_iface::*, info_provider_iface::*};
use crate::test_support::{require_native_api, require_unlinked_build};

static COMPLETION_DESTROY_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
static COMPLETION_ALREADY_DONE_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
static COMPLETION_SCHEDULE_FAILURE_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
static CANCEL_UPDATE_CALLS: AtomicUsize = AtomicUsize::new(0);
static ASYNC_UPDATE_CALLS: AtomicUsize = AtomicUsize::new(0);
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

struct PanickingUpdateInfoProvider;

impl InfoProvider for PanickingUpdateInfoProvider {
    fn update_file_info_full(&self, _operation: UpdateFileInfoOperation) -> OperationResult {
        panic!("info provider update callback panic");
    }
}

struct AsyncCancelInfoProvider;

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

unsafe extern "C" fn noop_update_complete() {}

fn test_update_complete_closure() -> *mut GClosure {
    unsafe { crate::gobject_ffi::g_cclosure_new(Some(noop_update_complete), ptr::null_mut(), None) }
}

fn opaque_native_file_info() -> FileInfo {
    let raw = unsafe {
        crate::gobject_ffi::g_object_new(crate::gobject_ffi::G_TYPE_OBJECT, ptr::null::<c_char>())
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
        operation_handle_for_update_result(OperationResult::InProgress, ptr::null_mut()).is_none()
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

#[test]
fn info_provider_iface_update_catches_provider_panics_with_real_native_inputs() {
    require_native_api!();

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

#[test]
fn info_provider_async_cancel_stress_uses_real_native_handles() {
    require_native_api!();

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

#[test]
fn documented_type_accessors_are_inert_in_no_link_mode() {
    require_unlinked_build!();

    assert_eq!(OperationResult::type_(), 0);
    assert_eq!(FileInfo::type_(), 0);
    assert_eq!(FileInfoHandle::type_(), 0);
    assert_eq!(InfoProviderHandle::type_(), 0);
}

#[test]
fn file_info_native_helpers_are_inert_in_no_link_mode() {
    require_unlinked_build!();

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

#[test]
fn documented_type_accessors_return_registered_gtypes() {
    require_native_api!();

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

    fn string_attribute(&self, _file_info: FileInfoHandle, attribute_name: &str) -> Option<String> {
        Some(format!("attribute:{attribute_name}"))
    }

    fn add_string_attribute(&self, _file_info: FileInfoHandle, attribute_name: &str, value: &str) {
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
