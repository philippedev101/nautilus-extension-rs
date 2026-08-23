//! Support for building without the native Nautilus extension library.
//!
//! `build.rs` sets the `nautilus_extension_rs_skip_link` cfg when the native
//! library is unavailable: on docs.rs, and on hosts where the Nautilus
//! extension development package is missing and
//! `NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG` allows the build to
//! continue anyway.
//!
//! In that configuration the C symbols cannot be referenced at all, so the
//! [`nautilus_api!`] macro below emits stub definitions with identical
//! signatures instead of the `extern "C"` declarations. Every stub ignores its
//! arguments and returns a neutral value, which lets the whole crate graph
//! compile and lets callers stay single-path. Callers that need to behave
//! differently should branch on [`crate::NATIVE_API_AVAILABLE`] at run time
//! rather than duplicating themselves behind a cfg.

/// Declares the Nautilus C API once and expands it for both build modes.
///
/// With the native library present this expands to the `extern "C"` block that
/// links against `libnautilus-extension`. Without it, each entry becomes a stub
/// function of the same signature returning [`UnlinkedDefault::unlinked_default`].
macro_rules! nautilus_api {
    (
        $(
            $(#[$attr:meta])*
            fn $name:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret:ty)?;
        )*
    ) => {
        #[cfg(not(nautilus_extension_rs_skip_link))]
        #[link(name = "nautilus-extension")]
        extern "C" {
            $(
                $(#[$attr])*
                pub fn $name($($arg: $arg_ty),*) $(-> $ret)?;
            )*
        }

        $(
            $(#[$attr])*
            ///
            /// # Safety
            ///
            /// This build has no native Nautilus library, so this is a stub
            /// that ignores its arguments and returns a neutral value. It keeps
            /// the signature of the linked function, whose safety requirements
            /// apply whenever the native library is present.
            #[cfg(nautilus_extension_rs_skip_link)]
            #[allow(unused_variables)]
            pub unsafe extern "C" fn $name($($arg: $arg_ty),*) $(-> $ret)? {
                $crate::unlinked::unlinked_default()
            }
        )*
    };
}

pub(crate) use nautilus_api;

/// The value a stub returns in place of calling the native library.
#[cfg(nautilus_extension_rs_skip_link)]
pub trait UnlinkedDefault {
    /// Returns the neutral value for this type.
    fn unlinked_default() -> Self;
}

/// Returns the neutral value for an unlinked call's return type.
#[cfg(nautilus_extension_rs_skip_link)]
pub fn unlinked_default<T: UnlinkedDefault>() -> T {
    T::unlinked_default()
}

#[cfg(nautilus_extension_rs_skip_link)]
mod impls {
    use super::UnlinkedDefault;
    use crate::NautilusOperationResult;

    impl UnlinkedDefault for () {
        fn unlinked_default() {}
    }

    impl<T> UnlinkedDefault for *mut T {
        fn unlinked_default() -> Self {
            std::ptr::null_mut()
        }
    }

    impl<T> UnlinkedDefault for *const T {
        fn unlinked_default() -> Self {
            std::ptr::null()
        }
    }

    /// Covers `gboolean` and `GFileType`, which are both `c_int`.
    impl UnlinkedDefault for libc::c_int {
        fn unlinked_default() -> Self {
            0
        }
    }

    /// Covers `GType`, which is `size_t`. Zero is `G_TYPE_INVALID`.
    impl UnlinkedDefault for libc::size_t {
        fn unlinked_default() -> Self {
            0
        }
    }

    impl UnlinkedDefault for NautilusOperationResult {
        fn unlinked_default() -> Self {
            NautilusOperationResult::NautilusOperationFailed
        }
    }
}
