// Tests deliberately build wrappers around bogus or null pointers to exercise the
// guards that reject them. A per-block safety comment there would only restate the
// name of the test, so the lint is off for test code and on everywhere else.
#![allow(clippy::undocumented_unsafe_blocks)]

use super::iface::*;
use super::*;
use crate::test_support::{require_native_api, require_unlinked_build};
use std::sync::atomic::{AtomicUsize, Ordering};

static GET_MODELS_CALLS: AtomicUsize = AtomicUsize::new(0);

struct RoutedPropertiesModelProvider;

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

#[test]
fn properties_model_provider_iface_trampoline_routes_to_registered_impl() {
    require_unlinked_build!();

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

#[test]
fn documented_type_accessors_are_inert_in_no_link_mode() {
    require_unlinked_build!();

    assert_eq!(PropertiesItemObject::type_(), 0);
    assert_eq!(PropertiesModelObject::type_(), 0);
    assert_eq!(PropertiesModelProviderHandle::type_(), 0);
}

#[test]
fn documented_type_accessors_return_registered_gtypes() {
    require_native_api!();

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

#[test]
fn native_object_construction_is_inert_in_no_link_mode() {
    require_unlinked_build!();

    assert!(PropertiesItemObject::new("Name", "Value").is_none());
    assert!(PropertiesItem::new("Name", "Value").to_object().is_none());

    let model = PropertiesModel::new("Section", vec![PropertiesItem::new("Name", "Value")]);

    assert!(model.to_object().is_none());
    assert!(model.to_raw().is_none());
}

#[test]
fn native_object_construction_round_trips_with_the_native_library() {
    require_native_api!();

    let item = PropertiesItemObject::new("Name", "Value").expect("item should be constructible");

    assert_eq!(item.name().as_deref(), Some("Name"));
    assert_eq!(item.value().as_deref(), Some("Value"));

    let model = PropertiesModel::new("Section", vec![PropertiesItem::new("Row", "Value")])
        .to_object()
        .expect("model should be constructible");

    assert_eq!(model.title().as_deref(), Some("Section"));
    assert_eq!(model.items().len(), 1);
}

#[test]
fn accessors_on_a_null_object_never_reach_glib() {
    // Reaching GLib with a null object logs a critical, and the test run makes
    // criticals fatal, so this aborts if a guard is ever dropped.
    let item = PropertiesItemObject {
        raw: ptr::null_mut(),
    };

    assert_eq!(item.name(), None);
    assert_eq!(item.value(), None);

    let model = PropertiesModelObject {
        raw: ptr::null_mut(),
    };

    assert_eq!(model.title(), None);
    assert!(!model.set_title("Section"));
    assert!(model.model().is_none());
    assert!(model.items().is_empty());
}
