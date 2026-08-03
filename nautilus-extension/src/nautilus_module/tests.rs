use super::*;
use crate::{
    Column, ColumnProvider, FileInfo, FileInfoImpl, InfoProvider, MenuItem, MenuItemObject,
    MenuProvider, PropertiesModel, PropertiesModelProvider,
};

struct TestColumnProvider;

impl ColumnProvider for TestColumnProvider {
    fn get_columns(&self) -> Vec<Column> {
        Vec::new()
    }
}

struct TestInfoProvider;

impl InfoProvider for TestInfoProvider {}

struct TestFileInfoImpl;

impl FileInfoImpl for TestFileInfoImpl {}

struct TestMenuProvider;

impl MenuProvider for TestMenuProvider {
    fn get_file_items(&self, _files: &[FileInfo]) -> Vec<MenuItem> {
        Vec::new()
    }
}

struct TestPropertiesModelProvider;

impl PropertiesModelProvider for TestPropertiesModelProvider {
    fn get_models(&self, _files: &[FileInfo]) -> Vec<PropertiesModel> {
        Vec::new()
    }
}

#[test]
fn module_starts_without_registered_interfaces() {
    let module = NautilusModule::new(ptr::null_mut(), "RustTestModule");

    assert_eq!(module.name.as_ref(), "RustTestModule");
    assert!(module.column_provider_iface_infos.is_empty());
    assert!(module.file_info_iface_infos.is_empty());
    assert!(module.info_provider_iface_infos.is_empty());
    assert!(module.menu_provider_iface_infos.is_empty());
    assert!(module.properties_model_provider_iface_infos.is_empty());
}

#[test]
fn try_add_provider_methods_record_interface_info() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_item_activate_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();

    let mut module = NautilusModule::new(ptr::null_mut(), "RustTestProviders");

    module
        .try_add_column_provider(TestColumnProvider)
        .unwrap()
        .try_add_file_info(TestFileInfoImpl)
        .unwrap()
        .try_add_info_provider(TestInfoProvider)
        .unwrap()
        .try_add_menu_provider(TestMenuProvider)
        .unwrap()
        .try_add_properties_model_provider(TestPropertiesModelProvider)
        .unwrap();

    assert_eq!(module.column_provider_iface_infos.len(), 1);
    assert_eq!(module.file_info_iface_infos.len(), 1);
    assert_eq!(module.info_provider_iface_infos.len(), 1);
    assert_eq!(module.menu_provider_iface_infos.len(), 1);
    assert_eq!(module.properties_model_provider_iface_infos.len(), 1);

    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_item_activate_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();
}

#[test]
fn module_error_describes_provider_limit() {
    let error = NautilusModuleError::TooManyProviders {
        provider_type: "menu",
        max: 10,
    };

    assert_eq!(
        error.to_string(),
        "too many Nautilus menu providers registered; this build supports 10"
    );

    let error = NautilusModuleError::DuplicateInterface {
        interface: "menu provider",
    };

    assert_eq!(
        error.to_string(),
        "Nautilus menu provider interface is already registered on this module type"
    );
}

#[test]
fn module_error_describes_new_registration_failures() {
    let invalid = NautilusModuleError::InvalidTypeName {
        type_name: Cow::Borrowed("Bad\0Type"),
    };
    let unavailable = NautilusModuleError::NativeApiUnavailable {
        api: "NautilusMenuItem subtype registration",
    };
    let failed = NautilusModuleError::TypeRegistrationFailed {
        type_name: Cow::Borrowed("ExampleMenuItem"),
    };

    assert_eq!(
        invalid.to_string(),
        "invalid GType name for Nautilus extension type: \"Bad\\0Type\""
    );
    assert_eq!(
        unavailable.to_string(),
        "Nautilus native API is unavailable in this build: NautilusMenuItem subtype registration"
    );
    assert_eq!(
        failed.to_string(),
        "failed to register Nautilus extension GType: ExampleMenuItem"
    );
}

#[test]
fn menu_item_type_registration_rejects_invalid_type_names_before_reserving_slot() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::menu_provider::reset_menu_item_activate_state();

    let mut module = NautilusModule::new(ptr::null_mut(), "RustInvalidMenuItemType");
    let error = module
        .try_register_menu_item_type("Bad\0Type", |_: &MenuItemObject| {})
        .unwrap_err();

    assert_eq!(
        error,
        NautilusModuleError::InvalidTypeName {
            type_name: Cow::Borrowed("Bad\0Type"),
        }
    );
    assert_eq!(take_next_menu_item_class_index(), Some(0));

    crate::menu_provider::reset_menu_item_activate_state();
}

#[cfg(nautilus_extension_rs_skip_link)]
#[test]
fn menu_item_type_registration_reports_unavailable_native_api_in_no_link_mode() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::menu_provider::reset_menu_item_activate_state();

    let mut module = NautilusModule::new(ptr::null_mut(), "RustUnavailableMenuItemType");
    let error = module
        .try_register_menu_item_type("RustUnavailableMenuItem", |_: &MenuItemObject| {})
        .unwrap_err();

    assert_eq!(
        error,
        NautilusModuleError::NativeApiUnavailable {
            api: "NautilusMenuItem subtype registration",
        }
    );
    assert!(!crate::menu_provider::menu_item_activate_slot_is_set(0));
    assert_eq!(take_next_menu_item_class_index(), Some(0));

    crate::menu_provider::reset_menu_item_activate_state();
}

#[cfg(nautilus_extension_rs_skip_link)]
#[test]
fn register_is_inert_and_releases_slots_in_no_link_mode() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();

    {
        let mut module = NautilusModule::new(ptr::null_mut(), "RustNoLinkRegister");
        module
            .try_add_column_provider(TestColumnProvider)
            .unwrap()
            .try_add_file_info(TestFileInfoImpl)
            .unwrap()
            .try_add_info_provider(TestInfoProvider)
            .unwrap()
            .try_add_menu_provider(TestMenuProvider)
            .unwrap()
            .try_add_properties_model_provider(TestPropertiesModelProvider)
            .unwrap();

        assert_eq!(module.register(), 0);
        assert!(!module.registered.get());
    }

    assert!(!crate::column_provider::column_provider_slot_is_set(0));
    assert!(!crate::info_provider::file_info_impl_slot_is_set(0));
    assert!(!crate::info_provider::info_provider_slot_is_set(0));
    assert!(!crate::menu_provider::menu_provider_slot_is_set(0));
    assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(0));

    assert_eq!(take_next_column_provider_iface_index(), Some(0));
    assert_eq!(take_next_file_info_iface_index(), Some(0));
    assert_eq!(take_next_info_provider_iface_index(), Some(0));
    assert_eq!(take_next_menu_provider_iface_index(), Some(0));
    assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();
}

#[test]
fn unregistered_module_drop_releases_reserved_provider_slots() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_item_activate_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();

    {
        let mut module = NautilusModule::new(ptr::null_mut(), "RustRollbackProviders");
        module
            .try_add_column_provider(TestColumnProvider)
            .unwrap()
            .try_add_file_info(TestFileInfoImpl)
            .unwrap()
            .try_add_info_provider(TestInfoProvider)
            .unwrap()
            .try_add_menu_provider(TestMenuProvider)
            .unwrap()
            .try_add_properties_model_provider(TestPropertiesModelProvider)
            .unwrap();

        assert!(crate::column_provider::column_provider_slot_is_set(0));
        assert!(crate::info_provider::file_info_impl_slot_is_set(0));
        assert!(crate::info_provider::info_provider_slot_is_set(0));
        assert!(crate::menu_provider::menu_provider_slot_is_set(0));
        assert!(crate::properties_model_provider::properties_model_provider_slot_is_set(0));
    }

    assert!(!crate::column_provider::column_provider_slot_is_set(0));
    assert!(!crate::info_provider::file_info_impl_slot_is_set(0));
    assert!(!crate::info_provider::info_provider_slot_is_set(0));
    assert!(!crate::menu_provider::menu_item_activate_slot_is_set(0));
    assert!(!crate::menu_provider::menu_provider_slot_is_set(0));
    assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(0));

    assert_eq!(take_next_column_provider_iface_index(), Some(0));
    assert_eq!(take_next_file_info_iface_index(), Some(0));
    assert_eq!(take_next_info_provider_iface_index(), Some(0));
    assert_eq!(take_next_menu_item_class_index(), Some(0));
    assert_eq!(take_next_menu_provider_iface_index(), Some(0));
    assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_item_activate_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();
}

#[test]
fn unregistered_module_drop_reuses_out_of_order_provider_slots() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();

    let mut first = NautilusModule::new(ptr::null_mut(), "RustOutOfOrderProvidersFirst");
    first
        .try_add_column_provider(TestColumnProvider)
        .unwrap()
        .try_add_file_info(TestFileInfoImpl)
        .unwrap()
        .try_add_info_provider(TestInfoProvider)
        .unwrap()
        .try_add_menu_provider(TestMenuProvider)
        .unwrap()
        .try_add_properties_model_provider(TestPropertiesModelProvider)
        .unwrap();

    let mut second = NautilusModule::new(ptr::null_mut(), "RustOutOfOrderProvidersSecond");
    second
        .try_add_column_provider(TestColumnProvider)
        .unwrap()
        .try_add_file_info(TestFileInfoImpl)
        .unwrap()
        .try_add_info_provider(TestInfoProvider)
        .unwrap()
        .try_add_menu_provider(TestMenuProvider)
        .unwrap()
        .try_add_properties_model_provider(TestPropertiesModelProvider)
        .unwrap();

    drop(first);

    assert!(!crate::column_provider::column_provider_slot_is_set(0));
    assert!(!crate::info_provider::file_info_impl_slot_is_set(0));
    assert!(!crate::info_provider::info_provider_slot_is_set(0));
    assert!(!crate::menu_provider::menu_provider_slot_is_set(0));
    assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(0));

    let mut third = NautilusModule::new(ptr::null_mut(), "RustOutOfOrderProvidersThird");
    third
        .try_add_column_provider(TestColumnProvider)
        .unwrap()
        .try_add_file_info(TestFileInfoImpl)
        .unwrap()
        .try_add_info_provider(TestInfoProvider)
        .unwrap()
        .try_add_menu_provider(TestMenuProvider)
        .unwrap()
        .try_add_properties_model_provider(TestPropertiesModelProvider)
        .unwrap();

    assert!(crate::column_provider::column_provider_slot_is_set(0));
    assert!(crate::info_provider::file_info_impl_slot_is_set(0));
    assert!(crate::info_provider::info_provider_slot_is_set(0));
    assert!(crate::menu_provider::menu_provider_slot_is_set(0));
    assert!(crate::properties_model_provider::properties_model_provider_slot_is_set(0));
    assert!(crate::column_provider::column_provider_slot_is_set(1));
    assert!(crate::info_provider::file_info_impl_slot_is_set(1));
    assert!(crate::info_provider::info_provider_slot_is_set(1));
    assert!(crate::menu_provider::menu_provider_slot_is_set(1));
    assert!(crate::properties_model_provider::properties_model_provider_slot_is_set(1));

    drop(second);
    drop(third);

    assert_eq!(take_next_column_provider_iface_index(), Some(0));
    assert_eq!(take_next_file_info_iface_index(), Some(0));
    assert_eq!(take_next_info_provider_iface_index(), Some(0));
    assert_eq!(take_next_menu_provider_iface_index(), Some(0));
    assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();
}

#[test]
fn duplicate_provider_interfaces_are_rejected_without_reserving_slots() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();

    {
        let mut module = NautilusModule::new(ptr::null_mut(), "RustDuplicateProviders");

        module.try_add_column_provider(TestColumnProvider).unwrap();
        let error = match module.try_add_column_provider(TestColumnProvider) {
            Ok(_) => panic!("duplicate column provider unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::DuplicateInterface {
                interface: "column provider",
            }
        );
        assert!(!crate::column_provider::column_provider_slot_is_set(1));

        module.try_add_file_info(TestFileInfoImpl).unwrap();
        let error = match module.try_add_file_info(TestFileInfoImpl) {
            Ok(_) => panic!("duplicate file info interface unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::DuplicateInterface {
                interface: "file info",
            }
        );
        assert!(!crate::info_provider::file_info_impl_slot_is_set(1));

        module.try_add_info_provider(TestInfoProvider).unwrap();
        let error = match module.try_add_info_provider(TestInfoProvider) {
            Ok(_) => panic!("duplicate info provider unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::DuplicateInterface {
                interface: "info provider",
            }
        );
        assert!(!crate::info_provider::info_provider_slot_is_set(1));

        module.try_add_menu_provider(TestMenuProvider).unwrap();
        let error = match module.try_add_menu_provider(TestMenuProvider) {
            Ok(_) => panic!("duplicate menu provider unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::DuplicateInterface {
                interface: "menu provider",
            }
        );
        assert!(!crate::menu_provider::menu_provider_slot_is_set(1));

        module
            .try_add_properties_model_provider(TestPropertiesModelProvider)
            .unwrap();
        let error = match module.try_add_properties_model_provider(TestPropertiesModelProvider) {
            Ok(_) => panic!("duplicate properties model provider unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::DuplicateInterface {
                interface: "properties model provider",
            }
        );
        assert!(!crate::properties_model_provider::properties_model_provider_slot_is_set(1));
    }

    assert_eq!(take_next_column_provider_iface_index(), Some(0));
    assert_eq!(take_next_file_info_iface_index(), Some(0));
    assert_eq!(take_next_info_provider_iface_index(), Some(0));
    assert_eq!(take_next_menu_provider_iface_index(), Some(0));
    assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));

    crate::column_provider::reset_column_provider_state();
    crate::info_provider::reset_file_info_impl_state();
    crate::info_provider::reset_info_provider_state();
    crate::menu_provider::reset_menu_provider_state();
    crate::properties_model_provider::reset_properties_model_provider_state();
}

#[test]
fn failed_try_add_does_not_advance_column_provider_slot_counter() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::column_provider::reset_column_provider_state();

    {
        let mut modules = Vec::new();
        for index in 0..MAX_COLUMN_PROVIDERS {
            let mut module =
                NautilusModule::new(ptr::null_mut(), format!("RustColumnOverflow{index}"));
            module.try_add_column_provider(TestColumnProvider).unwrap();
            modules.push(module);
        }

        let mut module = NautilusModule::new(ptr::null_mut(), "RustColumnOverflowExtra");
        let error = match module.try_add_column_provider(TestColumnProvider) {
            Ok(_) => panic!("column provider overflow unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::TooManyProviders {
                provider_type: "column",
                max: MAX_COLUMN_PROVIDERS,
            }
        );

        while let Some(module) = modules.pop() {
            drop(module);
        }
    }

    assert_eq!(take_next_column_provider_iface_index(), Some(0));
    crate::column_provider::reset_column_provider_state();
}

#[test]
fn failed_try_add_does_not_advance_file_info_slot_counter() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::info_provider::reset_file_info_impl_state();

    {
        let mut modules = Vec::new();
        for index in 0..MAX_FILE_INFO_IMPLS {
            let mut module =
                NautilusModule::new(ptr::null_mut(), format!("RustFileInfoOverflow{index}"));
            module.try_add_file_info(TestFileInfoImpl).unwrap();
            modules.push(module);
        }

        let mut module = NautilusModule::new(ptr::null_mut(), "RustFileInfoOverflowExtra");
        let error = match module.try_add_file_info(TestFileInfoImpl) {
            Ok(_) => panic!("file info overflow unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::TooManyProviders {
                provider_type: "file info",
                max: MAX_FILE_INFO_IMPLS,
            }
        );

        while let Some(module) = modules.pop() {
            drop(module);
        }
    }

    assert_eq!(take_next_file_info_iface_index(), Some(0));
    crate::info_provider::reset_file_info_impl_state();
}

#[test]
fn failed_try_add_does_not_advance_info_provider_slot_counter() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::info_provider::reset_info_provider_state();

    {
        let mut modules = Vec::new();
        for index in 0..MAX_INFO_PROVIDERS {
            let mut module =
                NautilusModule::new(ptr::null_mut(), format!("RustInfoOverflow{index}"));
            module.try_add_info_provider(TestInfoProvider).unwrap();
            modules.push(module);
        }

        let mut module = NautilusModule::new(ptr::null_mut(), "RustInfoOverflowExtra");
        let error = match module.try_add_info_provider(TestInfoProvider) {
            Ok(_) => panic!("info provider overflow unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::TooManyProviders {
                provider_type: "info",
                max: MAX_INFO_PROVIDERS,
            }
        );

        while let Some(module) = modules.pop() {
            drop(module);
        }
    }

    assert_eq!(take_next_info_provider_iface_index(), Some(0));
    crate::info_provider::reset_info_provider_state();
}

#[test]
fn failed_try_add_does_not_advance_menu_provider_slot_counter() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::menu_provider::reset_menu_provider_state();

    {
        let mut modules = Vec::new();
        for index in 0..MAX_MENU_PROVIDERS {
            let mut module =
                NautilusModule::new(ptr::null_mut(), format!("RustMenuOverflow{index}"));
            module.try_add_menu_provider(TestMenuProvider).unwrap();
            modules.push(module);
        }

        let mut module = NautilusModule::new(ptr::null_mut(), "RustMenuOverflowExtra");
        let error = match module.try_add_menu_provider(TestMenuProvider) {
            Ok(_) => panic!("menu provider overflow unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::TooManyProviders {
                provider_type: "menu",
                max: MAX_MENU_PROVIDERS,
            }
        );

        while let Some(module) = modules.pop() {
            drop(module);
        }
    }

    assert_eq!(take_next_menu_provider_iface_index(), Some(0));
    crate::menu_provider::reset_menu_provider_state();
}

#[test]
fn failed_try_add_does_not_advance_properties_model_provider_slot_counter() {
    let _guard = crate::test_support::PROVIDER_STATE_LOCK
        .lock()
        .expect("provider-state test lock poisoned");
    crate::properties_model_provider::reset_properties_model_provider_state();

    {
        let mut modules = Vec::new();
        for index in 0..MAX_PROPERTIES_MODEL_PROVIDERS {
            let mut module =
                NautilusModule::new(ptr::null_mut(), format!("RustPropertiesOverflow{index}"));
            module
                .try_add_properties_model_provider(TestPropertiesModelProvider)
                .unwrap();
            modules.push(module);
        }

        let mut module = NautilusModule::new(ptr::null_mut(), "RustPropertiesOverflowExtra");
        let error = match module.try_add_properties_model_provider(TestPropertiesModelProvider) {
            Ok(_) => panic!("properties model provider overflow unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            NautilusModuleError::TooManyProviders {
                provider_type: "properties model",
                max: MAX_PROPERTIES_MODEL_PROVIDERS,
            }
        );

        while let Some(module) = modules.pop() {
            drop(module);
        }
    }

    assert_eq!(take_next_properties_model_provider_iface_index(), Some(0));
    crate::properties_model_provider::reset_properties_model_provider_state();
}

#[test]
fn module_type_conversion_accepts_single_optional_vec_and_array() {
    assert_eq!(IntoModuleTypes::into_module_types(7 as GType), vec![7]);
    assert!(IntoModuleTypes::into_module_types(0 as GType).is_empty());
    assert_eq!(
        IntoModuleTypes::into_module_types(Some(9 as GType)),
        vec![9]
    );
    assert!(IntoModuleTypes::into_module_types(None::<GType>).is_empty());
    assert_eq!(
        IntoModuleTypes::into_module_types(vec![1 as GType, 0 as GType, 2 as GType]),
        vec![1, 2]
    );
    assert_eq!(
        IntoModuleTypes::into_module_types([3 as GType, 0 as GType, 4 as GType]),
        vec![3, 4]
    );
}
