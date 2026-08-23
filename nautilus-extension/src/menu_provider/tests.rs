use super::*;
use super::{activation::*, menu_item_class::*, menu_provider_iface::*};
use crate::test_support::{require_native_api, require_unlinked_build};
use std::sync::atomic::Ordering;

static MENU_DESTROY_DROP_CALLS: AtomicUsize = AtomicUsize::new(0);
static MENU_GET_FILE_ITEMS_CALLS: AtomicUsize = AtomicUsize::new(0);

struct PanickingMenuDropPayload;

impl Drop for PanickingMenuDropPayload {
    fn drop(&mut self) {
        MENU_DESTROY_DROP_CALLS.fetch_add(1, Ordering::SeqCst);
        panic!("menu destroy callback payload drop panic");
    }
}

struct RoutedMenuProvider;

impl MenuProvider for RoutedMenuProvider {
    fn get_file_items(&self, files: &[FileInfo]) -> Vec<MenuItem> {
        assert!(files.is_empty());
        MENU_GET_FILE_ITEMS_CALLS.fetch_add(1, Ordering::SeqCst);
        Vec::new()
    }
}

struct PanickingMenuProvider;

impl MenuProvider for PanickingMenuProvider {
    fn get_file_items(&self, _files: &[FileInfo]) -> Vec<MenuItem> {
        panic!("menu provider callback panic");
    }
}

struct PanickingBackgroundMenuProvider;

impl MenuProvider for PanickingBackgroundMenuProvider {
    fn get_background_items(&self, _current_folder: &FileInfo) -> Vec<MenuItem> {
        panic!("menu provider background callback panic");
    }
}

struct PanickingMenuItemActivate;

impl MenuItemActivate for PanickingMenuItemActivate {
    fn activate(&self, _item: &MenuItemObject) {
        panic!("menu item activate callback panic");
    }
}

fn opaque_native_file_info() -> FileInfo {
    let raw = unsafe {
        crate::gobject_ffi::g_object_new(crate::gobject_ffi::G_TYPE_OBJECT, ptr::null::<c_char>())
    };

    unsafe { FileInfo::from_raw_full(raw as *mut NautilusFileInfo) }
        .expect("GObject should be constructible for opaque native FileInfo tests")
}

#[test]
fn menu_from_slice_clones_items() {
    let item = MenuItem::new("Example::item", "Example Item");
    let menu = Menu::from_slice(&[item]);

    assert_eq!(menu.menu_items.len(), 1);
    assert_eq!(menu.menu_items[0].name.as_ref(), "Example::item");
    assert_eq!(menu.menu_items[0].label.as_ref(), "Example Item");
}

#[test]
#[allow(deprecated)]
fn menu_item_builder_keeps_optional_state() {
    let item_type = unsafe { MenuItemType::from_raw(77) }.unwrap();
    let submenu = Menu::new(vec![MenuItem::new("Example::child", "Child")]);
    let item = MenuItem::new("Example::parent", "Parent")
        .with_menu_item_type(item_type)
        .with_tip("Tip")
        .with_icon("folder")
        .sensitive(false)
        .priority(false)
        .with_submenu(submenu)
        .on_activate(|_| {});

    assert_eq!(item.name.as_ref(), "Example::parent");
    assert_eq!(item.label.as_ref(), "Parent");
    assert_eq!(item.item_type, Some(item_type));
    assert_eq!(item.item_type(), Some(item_type));
    assert_eq!(item.tip.as_ref().map(|tip| tip.as_ref()), Some("Tip"));
    assert_eq!(item.icon.as_ref().map(|icon| icon.as_ref()), Some("folder"));
    assert!(!item.sensitive);
    assert_eq!(item.priority, Some(false));
    assert_eq!(
        item.submenu.as_ref().map(|menu| menu.menu_items.len()),
        Some(1)
    );
    assert!(item.activate_fn.is_some());
}

#[test]
fn menu_item_type_rejects_zero_and_preserves_raw_gtype() {
    assert!(unsafe { MenuItemType::from_raw(0) }.is_none());

    let item_type = unsafe { MenuItemType::from_raw(77) }.unwrap();

    assert_eq!(item_type.raw(), 77);
}

#[test]
fn menu_item_activate_trait_supports_closures() {
    let calls = AtomicUsize::new(0);
    let activate = |_: &MenuItemObject| {
        calls.fetch_add(1, Ordering::SeqCst);
    };
    let item = MenuItemObject {
        raw: ptr::null_mut(),
    };

    activate.activate(&item);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn menu_item_class_init_installs_activate_vfunc() {
    let mut class: NautilusMenuItemClass = unsafe { std::mem::zeroed() };

    unsafe {
        menu_item_class_init_0(
            &mut class as *mut NautilusMenuItemClass as gpointer,
            ptr::null_mut(),
        );
    }

    assert!(class.activate.is_some());
}

#[test]
fn menu_object_wrappers_reject_null_raw_pointers() {
    assert!(unsafe { MenuObject::from_raw_full(ptr::null_mut()) }.is_none());
    assert!(unsafe { MenuObject::from_raw_borrowed(ptr::null_mut()) }.is_none());
    assert!(unsafe { MenuItemObject::from_raw_full(ptr::null_mut()) }.is_none());
    assert!(unsafe { MenuItemObject::from_raw_borrowed(ptr::null_mut()) }.is_none());
    assert!(unsafe { MenuItemList::from_raw_full(ptr::null_mut()) }.is_none());
    assert!(unsafe { MenuProviderHandle::from_raw_full(ptr::null_mut()) }.is_none());
    assert!(unsafe { MenuProviderHandle::from_raw_borrowed(ptr::null_mut()) }.is_none());
}

#[test]
fn signal_handler_id_rejects_zero_and_preserves_raw_value() {
    assert_eq!(SignalHandlerId::from_raw(0), None);

    let signal_id = SignalHandlerId::from_raw(7).unwrap();
    assert_eq!(signal_id.raw(), 7);
}

#[test]
fn menu_provider_iface_trampoline_routes_file_items_to_registered_impl() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    reset_menu_provider_state();
    MENU_GET_FILE_ITEMS_CALLS.store(0, Ordering::SeqCst);

    set_menu_provider_0(Box::new(RoutedMenuProvider));

    let mut iface: NautilusMenuProviderIface = unsafe { std::mem::zeroed() };
    unsafe {
        menu_provider_iface_init_0(
            &mut iface as *mut NautilusMenuProviderIface as gpointer,
            ptr::null_mut(),
        );
    }

    let items = unsafe { (iface.get_file_items.unwrap())(ptr::null_mut(), ptr::null_mut()) };

    assert!(items.is_null());
    assert_eq!(MENU_GET_FILE_ITEMS_CALLS.load(Ordering::SeqCst), 1);
    assert!(iface.get_background_items.is_some());

    reset_menu_provider_state();
}

#[test]
fn menu_provider_iface_trampoline_catches_file_item_provider_panics() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    reset_menu_provider_state();

    set_menu_provider_0(Box::new(PanickingMenuProvider));

    let mut iface: NautilusMenuProviderIface = unsafe { std::mem::zeroed() };
    unsafe {
        menu_provider_iface_init_0(
            &mut iface as *mut NautilusMenuProviderIface as gpointer,
            ptr::null_mut(),
        );
    }

    let result = std::panic::catch_unwind(|| unsafe {
        (iface.get_file_items.unwrap())(ptr::null_mut(), ptr::null_mut())
    });

    assert_eq!(result.unwrap(), ptr::null_mut());

    reset_menu_provider_state();
}

#[test]
fn menu_provider_iface_trampoline_catches_background_provider_panics() {
    require_native_api!();

    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    reset_menu_provider_state();

    set_menu_provider_0(Box::new(PanickingBackgroundMenuProvider));

    let mut iface: NautilusMenuProviderIface = unsafe { std::mem::zeroed() };
    unsafe {
        menu_provider_iface_init_0(
            &mut iface as *mut NautilusMenuProviderIface as gpointer,
            ptr::null_mut(),
        );
    }

    let folder = opaque_native_file_info();
    let result = std::panic::catch_unwind(|| unsafe {
        (iface.get_background_items.unwrap())(ptr::null_mut(), folder.raw())
    });

    assert_eq!(result.unwrap(), ptr::null_mut());

    reset_menu_provider_state();
}

#[test]
fn documented_type_accessors_are_inert_in_no_link_mode() {
    require_unlinked_build!();

    assert_eq!(MenuObject::type_(), 0);
    assert_eq!(MenuItemObject::type_(), 0);
    assert_eq!(MenuProviderHandle::type_(), 0);
}

#[test]
fn optional_submenu_assignment_is_inert_in_no_link_mode() {
    require_unlinked_build!();

    let item = MenuItemObject {
        raw: ptr::null_mut(),
    };
    let submenu = MenuObject {
        raw: ptr::null_mut(),
    };

    assert!(!item.set_optional_submenu(Some(&submenu)));
    assert!(!item.set_optional_submenu(None));
    assert!(!item.clear_submenu());
}

#[test]
fn documented_type_accessors_return_registered_gtypes() {
    require_native_api!();

    assert_ne!(MenuObject::type_(), 0);
    assert_ne!(MenuItemObject::type_(), 0);
    assert_ne!(MenuProviderHandle::type_(), 0);
}

#[test]
fn signal_destroy_callbacks_catch_panicking_payload_drops() {
    MENU_DESTROY_DROP_CALLS.store(0, Ordering::SeqCst);

    let payload = PanickingMenuDropPayload;
    let provider_data = Box::new(MenuProviderItemsUpdatedData {
        callback: Box::new(move |_| {
            let _ = &payload;
        }),
    });
    unsafe {
        destroy_menu_provider_items_updated_data(
            Box::into_raw(provider_data) as gpointer,
            ptr::null_mut(),
        );
    }

    let payload = PanickingMenuDropPayload;
    let item_data = Box::new(MenuItemObjectActivateData {
        callback: Box::new(move |_| {
            let _ = &payload;
        }),
    });
    unsafe {
        destroy_menu_item_object_activate_data(
            Box::into_raw(item_data) as gpointer,
            ptr::null_mut(),
        );
    }

    let payload = PanickingMenuDropPayload;
    let activate_data = Box::new(ActivateData {
        activate_fn: Arc::new(move |_| {
            let _ = &payload;
        }),
        target: MenuActivationTarget::Files(Vec::new()),
    });
    unsafe {
        destroy_activate_data(Box::into_raw(activate_data) as gpointer, ptr::null_mut());
    }

    assert_eq!(MENU_DESTROY_DROP_CALLS.load(Ordering::SeqCst), 3);
}

#[test]
fn signal_trampolines_catch_panicking_callbacks() {
    require_native_api!();

    let file = opaque_native_file_info();
    let item = MenuItemObject::new("RustValidation::signal", "Signal").unwrap();

    let provider_data = Box::new(MenuProviderItemsUpdatedData {
        callback: Box::new(|_| panic!("items-updated signal callback panic")),
    });
    let provider_data = Box::into_raw(provider_data);
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        menu_provider_items_updated_trampoline(
            item.raw() as *mut NautilusMenuProvider,
            provider_data as gpointer,
        );
    }));
    assert!(result.is_ok());
    unsafe {
        drop(Box::from_raw(provider_data));
    }

    let item_data = Box::new(MenuItemObjectActivateData {
        callback: Box::new(|_| panic!("menu item object activate callback panic")),
    });
    let item_data = Box::into_raw(item_data);
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        menu_item_object_activate_trampoline(item.raw(), item_data as gpointer);
    }));
    assert!(result.is_ok());
    unsafe {
        drop(Box::from_raw(item_data));
    }

    let activate_data = Box::new(ActivateData {
        activate_fn: Arc::new(|_| panic!("menu activation callback panic")),
        target: MenuActivationTarget::Background(file),
    });
    let activate_data = Box::into_raw(activate_data);
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        menu_item_activate_trampoline(item.raw(), activate_data as gpointer);
    }));
    assert!(result.is_ok());
    unsafe {
        drop(Box::from_raw(activate_data));
    }
}

#[test]
fn menu_item_activate_vfunc_catches_provider_panics() {
    require_native_api!();

    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    reset_menu_item_activate_state();

    set_menu_item_activate_0(Box::new(PanickingMenuItemActivate));
    let item = MenuItemObject::new("RustValidation::activate", "Activate").unwrap();

    let result = std::panic::catch_unwind(|| unsafe {
        menu_item_activate_0(item.raw());
    });

    assert!(result.is_ok());

    reset_menu_item_activate_state();
}

#[test]
fn set_submenu_accepts_borrowed_submenu_wrapper() {
    require_native_api!();

    let item = MenuItemObject::new("Example::parent", "Parent").unwrap();
    let submenu = MenuObject::new().unwrap();

    item.set_submenu(&submenu);

    assert!(item.submenu().is_some());
    assert!(submenu.get_items().is_empty());
}

#[test]
fn optional_submenu_assignment_can_attach_submenus() {
    require_native_api!();

    let item = MenuItemObject::new("Example::parent", "Parent").unwrap();
    let submenu = MenuObject::new().unwrap();

    assert!(item.set_optional_submenu(Some(&submenu)));
    assert!(item.submenu().is_some());

    assert!(item.set_optional_submenu(Some(&submenu)));
    assert!(item.submenu().is_some());

    assert!(!item.set_optional_submenu(None));
    assert!(!item.clear_submenu());
}

#[test]
fn reset_menu_provider_state_allows_slot_reuse() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    reset_menu_provider_state();

    for expected_index in 0..MAX_MENU_PROVIDERS {
        assert_eq!(take_next_menu_provider_iface_index(), Some(expected_index));
    }

    assert_eq!(take_next_menu_provider_iface_index(), None);

    reset_menu_provider_state();

    assert_eq!(take_next_menu_provider_iface_index(), Some(0));

    reset_menu_provider_state();
}

#[test]
fn reset_menu_item_activate_state_allows_slot_reuse() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    reset_menu_item_activate_state();

    for expected_index in 0..MAX_MENU_ITEM_ACTIVATORS {
        assert_eq!(take_next_menu_item_class_index(), Some(expected_index));
    }

    assert_eq!(take_next_menu_item_class_index(), None);

    reset_menu_item_activate_state();

    assert_eq!(take_next_menu_item_class_index(), Some(0));

    reset_menu_item_activate_state();
}

#[test]
fn menu_item_activate_slots_reuse_out_of_order_releases() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    reset_menu_item_activate_state();

    let first = take_next_menu_item_class_index().unwrap();
    let second = take_next_menu_item_class_index().unwrap();

    assert_eq!(first, 0);
    assert_eq!(second, 1);

    release_menu_item_class_index(first);

    let reused = take_next_menu_item_class_index().unwrap();
    assert_eq!(reused, first);

    release_menu_item_class_index(second);
    release_menu_item_class_index(reused);

    assert_eq!(take_next_menu_item_class_index(), Some(0));

    reset_menu_item_activate_state();
}

#[test]
#[allow(deprecated)]
fn native_object_construction_is_inert_in_no_link_mode() {
    require_unlinked_build!();

    let target = MenuActivationTarget::Files(Vec::new());

    assert!(MenuObject::new().is_none());
    assert!(MenuItemObject::new("Example::item", "Item").is_none());
    assert!(
        MenuItemObject::new_full("Example::item", "Item", Some("Tip"), Some("folder")).is_none()
    );
    assert!(MenuItem::new("Example::item", "Item")
        .to_object(&target)
        .is_none());
    assert!(Menu::new(vec![MenuItem::new("Example::item", "Item")])
        .to_object(&target)
        .is_none());
    assert!(Menu::new(vec![MenuItem::new("Example::item", "Item")])
        .to_raw(&target)
        .is_null());
}

#[test]
#[allow(deprecated)]
fn menu_item_accessors_on_a_null_object_never_reach_glib() {
    // Reaching GLib with a null object logs a critical, and the test run makes
    // criticals fatal, so this aborts if a guard is ever dropped.
    let item = MenuItemObject {
        raw: ptr::null_mut(),
    };

    assert_eq!(item.name(), None);
    assert_eq!(item.label(), None);
    assert_eq!(item.tip(), None);
    assert_eq!(item.icon(), None);
    assert!(!item.sensitive());
    assert!(!item.priority());
    assert!(item.submenu().is_none());

    assert!(!item.set_label("Label"));
    assert!(!item.set_tip(Some("Tip")));
    assert!(!item.set_tip(None));
    assert!(!item.set_icon(Some("folder")));
    assert!(!item.set_sensitive(true));
    assert!(!item.set_priority(true));
}
