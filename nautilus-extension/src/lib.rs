#![deny(bare_trait_objects)]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(unsafe_op_in_unsafe_fn)]
//! Rust bindings for Nautilus API 4 extension modules.
//!
//! This crate wraps the extension ABI used by Nautilus 43 and newer, where the
//! extension API moved to GTK 4-era `libnautilus-extension.so.4`. The "4" here
//! refers to the Nautilus extension API and ABI, not to the GNOME Files
//! application version.
//!
//! Extension crates normally implement one or more provider traits, register
//! them with [`NautilusModule`], build as a `cdylib`, and export Nautilus'
//! required C entry points through [`nautilus_module!`].
//!
//! # Provider APIs
//!
//! The high-level provider traits correspond to the current Nautilus API 4
//! extension surfaces:
//!
//! * [`ColumnProvider`] declares custom list-view columns.
//! * [`InfoProvider`] fills extra attributes, including asynchronous
//!   `update_file_info_full` operations through [`PendingUpdate`].
//! * [`MenuProvider`] adds file and background context-menu items.
//! * [`PropertiesModelProvider`] adds model-backed sections to the Properties
//!   dialog.
//!
//! The crate also exposes owned wrappers for API objects such as [`FileInfo`],
//! [`Column`], [`Menu`], [`MenuItem`], [`PropertiesModel`], and
//! [`PropertiesItem`].
//!
//! # Async model
//!
//! Nautilus API 4 has a native asynchronous completion contract for
//! [`InfoProvider`] only. Use [`InfoProvider::update_file_info_full`] when file
//! metadata work may block or take noticeable time, return
//! [`OperationResult::InProgress`], check [`OperationHandle::is_cancelled`] from
//! worker code, and finish once with [`UpdateCompletion::complete`] or
//! [`UpdateCompletion::complete_with`].
//!
//! [`ColumnProvider`], [`MenuProvider`], and [`PropertiesModelProvider`] remain
//! synchronous because Nautilus requests their return values immediately. Keep
//! those callbacks cheap, read from caches where needed, and use
//! [`MenuProviderHandle::emit_items_updated_signal`] when cached menu state
//! changes.
//!
//! This crate does not impose Tokio, async-std, or any other general async
//! runtime. Downstream extensions can use their preferred worker pool or
//! runtime and hand results back through the Nautilus completion API.
//!
//! # Example
//!
//! A column extension usually combines [`ColumnProvider`] with [`InfoProvider`].
//! The column `attribute` must match the attribute later set on each
//! [`FileInfo`].
//!
//! ```no_run
//! use nautilus_extension::{
//!     Column, ColumnProvider, FileInfo, InfoProvider, NautilusModule,
//!     OperationResult, UpdateFileInfoOperation,
//! };
//! use nautilus_extension::gobject_ffi::GTypeModule;
//!
//! const ATTRIBUTE: &str = "example_attribute";
//!
//! struct Provider;
//!
//! impl ColumnProvider for Provider {
//!     fn get_columns(&self) -> Vec<Column> {
//!         vec![Column::new(
//!             "Example::metadata",
//!             ATTRIBUTE,
//!             "Example",
//!             "Example metadata",
//!         )]
//!     }
//! }
//!
//! impl InfoProvider for Provider {
//!     fn update_file_info_full(&self, operation: UpdateFileInfoOperation) -> OperationResult {
//!         operation
//!             .file_info()
//!             .add_string_attribute(ATTRIBUTE, "example value");
//!         OperationResult::Complete
//!     }
//! }
//!
//! fn register(module: *mut GTypeModule) -> nautilus_extension::glib_ffi::GType {
//!     let mut module = NautilusModule::new(module, "RustExampleExtension");
//!     module
//!         .add_column_provider(Provider)
//!         .add_info_provider(Provider)
//!         .register()
//! }
//!
//! nautilus_extension::nautilus_module!(register);
//! ```
//!
//! # Nautilus 3
//!
//! The primary API is Nautilus API 4. Obsolete GTK 3-era interfaces such as
//! `LocationWidgetProvider`, GTK-widget property pages, and menu provider
//! callbacks with window arguments are intentionally not reintroduced here.
//!
//! # Panics and threading
//!
//! Nautilus loads extension shared objects in-process. Provider trampolines in
//! this crate catch panics before returning to C, but extension code should not
//! use panics for normal control flow. Avoid blocking provider callbacks; for
//! expensive file-info work, use a bounded worker queue or downstream runtime
//! and finish through [`UpdateCompletion::complete_with`].

pub extern crate gio_sys as gio_ffi;
pub extern crate glib_sys as glib_ffi;
pub extern crate gobject_sys as gobject_ffi;
#[macro_use]
extern crate lazy_static;
pub extern crate libc;
pub extern crate nautilus_extension_sys as nautilus_ffi;

/// Whether this build links against the native Nautilus extension library.
///
/// This is `false` only when the library was unavailable at build time, such
/// as on docs.rs, in which case every Nautilus call is a stub that returns a
/// neutral value and the provider registration entry points report
/// [`NautilusModuleError::NativeApiUnavailable`].
pub use crate::nautilus_ffi::NATIVE_API_AVAILABLE;

pub use crate::column_provider::{
    Column, ColumnObject, ColumnProvider, ColumnProviderHandle, ColumnSortOrder,
};
pub use crate::info_provider::{
    FileInfo, FileInfoHandle, FileInfoImpl, FileInfoList, InfoProvider, InfoProviderHandle,
    OperationHandle, OperationResult, OwnedGObject, PendingUpdate, UpdateCompleteCallback,
    UpdateCompletion, UpdateFileInfoOperation,
};
pub use crate::menu_provider::{
    Menu, MenuActivation, MenuActivationTarget, MenuItem, MenuItemActivate, MenuItemList,
    MenuItemObject, MenuItemType, MenuObject, MenuProvider, MenuProviderHandle, SignalHandlerId,
};
pub use crate::nautilus_module::{IntoModuleTypes, NautilusModule, NautilusModuleError};
pub use crate::properties_model_provider::{
    PropertiesItem, PropertiesItemObject, PropertiesModel, PropertiesModelObject,
    PropertiesModelProvider, PropertiesModelProviderHandle,
};
pub use lazy_static::lazy_static;

/// Column-provider traits and wrappers.
pub mod column_provider;
mod gobject_utils;
/// File-info traits, wrappers, and asynchronous update helpers.
pub mod info_provider;
/// Context-menu provider traits and menu wrappers.
pub mod menu_provider;
mod nautilus_module;
/// Properties-model provider traits and wrappers.
pub mod properties_model_provider;
mod slot_allocator;
mod translate;

/// Exports the C ABI entry points required by Nautilus.
///
/// The registration function is called from `nautilus_module_initialize` and
/// receives Nautilus' raw `GTypeModule` pointer. It may return one `GType`, an
/// `Option<GType>`, a `Vec<GType>`, or a fixed-size array of `GType` values.
/// The macro also exports `nautilus_module_list_types` and
/// `nautilus_module_shutdown`.
///
/// # Panics
///
/// Panics from the registration function are caught before control returns to
/// Nautilus. A panic leaves the exported type list empty.
///
/// # Example
///
/// ```no_run
/// use nautilus_extension::{NautilusModule, MenuProvider};
/// use nautilus_extension::gobject_ffi::GTypeModule;
///
/// struct Provider;
///
/// impl MenuProvider for Provider {}
///
/// fn register(module: *mut GTypeModule) -> nautilus_extension::glib_ffi::GType {
///     let mut module = NautilusModule::new(module, "RustMenuExample");
///     module.add_menu_provider(Provider).register()
/// }
///
/// nautilus_extension::nautilus_module!(register);
/// ```
#[macro_export]
macro_rules! nautilus_module {
    ($register_fn:path) => {
        $crate::lazy_static! {
            static ref MODULE_TYPE_LIST: std::sync::Mutex<Vec<$crate::glib_ffi::GType>> =
                std::sync::Mutex::new(Vec::new());
        }

        #[no_mangle]
        pub unsafe extern "C" fn nautilus_module_initialize(
            module: *mut $crate::gobject_ffi::GTypeModule,
        ) {
            let initialized = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Ok(mut module_types) = MODULE_TYPE_LIST.lock() {
                    module_types.clear();
                }

                $crate::column_provider::reset_column_provider_state();
                $crate::info_provider::reset_file_info_impl_state();
                $crate::info_provider::reset_info_provider_state();
                $crate::menu_provider::reset_menu_item_activate_state();
                $crate::menu_provider::reset_menu_provider_state();
                $crate::properties_model_provider::reset_properties_model_provider_state();

                let module_types =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $register_fn(module)));

                if let Ok(registered_types) = module_types {
                    if let Ok(mut module_types) = MODULE_TYPE_LIST.lock() {
                        module_types
                            .extend($crate::IntoModuleTypes::into_module_types(registered_types));
                    }
                } else {
                    eprintln!("nautilus-extension-rs: module initialization panicked");
                    $crate::column_provider::reset_column_provider_state();
                    $crate::info_provider::reset_file_info_impl_state();
                    $crate::info_provider::reset_info_provider_state();
                    $crate::menu_provider::reset_menu_item_activate_state();
                    $crate::menu_provider::reset_menu_provider_state();
                    $crate::properties_model_provider::reset_properties_model_provider_state();
                }
            }));

            if initialized.is_err() {
                eprintln!("nautilus-extension-rs: module initialization panicked");
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if let Ok(mut module_types) = MODULE_TYPE_LIST.lock() {
                        module_types.clear();
                    }

                    $crate::column_provider::reset_column_provider_state();
                    $crate::info_provider::reset_file_info_impl_state();
                    $crate::info_provider::reset_info_provider_state();
                    $crate::menu_provider::reset_menu_item_activate_state();
                    $crate::menu_provider::reset_menu_provider_state();
                    $crate::properties_model_provider::reset_properties_model_provider_state();
                }));
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn nautilus_module_list_types(
            types: *mut *const $crate::glib_ffi::GType,
            num_types: *mut $crate::libc::c_int,
        ) {
            if let Ok(module_types) = MODULE_TYPE_LIST.lock() {
                if !types.is_null() {
                    unsafe {
                        *types = module_types.as_ptr();
                    }
                }
                if !num_types.is_null() {
                    unsafe {
                        *num_types = module_types.len() as $crate::libc::c_int;
                    }
                }
            } else {
                if !types.is_null() {
                    unsafe {
                        *types = std::ptr::null();
                    }
                }
                if !num_types.is_null() {
                    unsafe {
                        *num_types = 0;
                    }
                }
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn nautilus_module_shutdown() {
            let shutdown = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Ok(mut module_types) = MODULE_TYPE_LIST.lock() {
                    module_types.clear();
                }

                $crate::column_provider::reset_column_provider_state();
                $crate::info_provider::reset_file_info_impl_state();
                $crate::info_provider::reset_info_provider_state();
                $crate::menu_provider::reset_menu_item_activate_state();
                $crate::menu_provider::reset_menu_provider_state();
                $crate::properties_model_provider::reset_properties_model_provider_state();
            }));

            if shutdown.is_err() {
                eprintln!("nautilus-extension-rs: module shutdown panicked");
            }
        }
    };
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::glib_ffi::GType;
    use crate::gobject_ffi::GTypeModule;
    use std::ptr;

    pub static PROVIDER_STATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Ends the calling test early unless this build links against Nautilus.
    ///
    /// Tests that drive real `NautilusColumn`, `NautilusMenu`, or
    /// `NautilusPropertiesModel` objects cannot run when the native library
    /// is missing, because every constructor is then a stub returning null.
    macro_rules! require_native_api {
        () => {
            if !$crate::NATIVE_API_AVAILABLE {
                return;
            }
        };
    }

    /// Ends the calling test early unless this build is missing Nautilus.
    ///
    /// The counterpart of [`require_native_api!`], for the tests that pin how
    /// the wrappers behave once every Nautilus call is an inert stub.
    macro_rules! require_unlinked_build {
        () => {
            if $crate::NATIVE_API_AVAILABLE {
                return;
            }
        };
    }

    pub(crate) use {require_native_api, require_unlinked_build};

    fn panicking_register(_module: *mut GTypeModule) -> GType {
        panic!("module registration panic");
    }

    crate::nautilus_module!(panicking_register);

    #[test]
    fn module_initialize_catches_registration_panic() {
        let _guard = PROVIDER_STATE_LOCK
            .lock()
            .expect("provider-state test lock poisoned");

        unsafe {
            nautilus_module_initialize(ptr::null_mut());
        }

        let mut types: *const GType = ptr::null();
        let mut num_types = -1;

        unsafe {
            nautilus_module_list_types(&mut types, &mut num_types);
            nautilus_module_shutdown();
        }

        assert_eq!(num_types, 0);
    }
}
