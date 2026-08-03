use super::*;
use std::mem;

#[test]
fn documented_class_structs_have_the_expected_prefix_layout() {
    assert_eq!(
        mem::size_of::<NautilusColumnClass>(),
        mem::size_of::<GObjectClass>()
    );
    assert_eq!(
        mem::size_of::<NautilusMenuClass>(),
        mem::size_of::<GObjectClass>()
    );
    assert_eq!(
        mem::size_of::<NautilusPropertiesItemClass>(),
        mem::size_of::<GObjectClass>()
    );
    assert_eq!(
        mem::size_of::<NautilusPropertiesModelClass>(),
        mem::size_of::<GObjectClass>()
    );
    assert_eq!(
        mem::size_of::<NautilusMenuItemClass>(),
        mem::size_of::<GObjectClass>() + mem::size_of::<usize>()
    );
}

#[test]
fn documented_interface_aliases_match_existing_iface_structs() {
    assert_eq!(
        mem::size_of::<NautilusColumnProviderInterface>(),
        mem::size_of::<NautilusColumnProviderIface>()
    );
    assert_eq!(
        mem::size_of::<NautilusInfoProviderInterface>(),
        mem::size_of::<NautilusInfoProviderIface>()
    );
    assert_eq!(
        mem::size_of::<NautilusMenuProviderInterface>(),
        mem::size_of::<NautilusMenuProviderIface>()
    );
    assert_eq!(
        mem::size_of::<NautilusPropertiesModelProviderInterface>(),
        mem::size_of::<NautilusPropertiesModelProviderIface>()
    );
}

#[test]
fn operation_result_values_match_documented_c_enum() {
    assert_eq!(NautilusOperationResult::NautilusOperationComplete as i32, 0);
    assert_eq!(NautilusOperationResult::NautilusOperationFailed as i32, 1);
    assert_eq!(
        NautilusOperationResult::NautilusOperationInProgress as i32,
        2
    );
}
