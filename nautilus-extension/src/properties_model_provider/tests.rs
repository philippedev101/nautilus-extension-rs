use super::iface::*;
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
    let mut model = PropertiesModel::new("Example", vec![PropertiesItem::new("Name length", "12")]);

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
    assert!(unsafe { PropertiesModelProviderHandle::from_raw_borrowed(ptr::null_mut()) }.is_none());
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
