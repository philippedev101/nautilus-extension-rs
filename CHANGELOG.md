# Changelog

## 0.9.0 (unreleased)

* Ported the library to the Nautilus API 4 / GTK 4-era extension ABI used by
  Nautilus 43 and newer.
* Require Rust 1.92, up from 1.57, and check it in CI.
* Added wrappers for `PropertiesModelProvider`, model-backed properties items
  and models, asynchronous `InfoProvider` completion, operation handles,
  provider handles, and modern menu APIs.
* Removed Nautilus 3 / GTK 3-only property page and location widget APIs from
  the primary API surface.
* Added neutral column, menu, and properties model examples that build as
  `cdylib` extension modules for `extensions-4`.
* Added validation scripts, fuzz/sanitizer smoke tests, docs.rs support,
  API coverage tracking, and release documentation.
* Replaced the per-call `nautilus_extension_rs_skip_link` cfg pairs with a
  single set of stub definitions in `nautilus-extension-sys`, so the wrapper
  source has one code path for both linked and unlinked builds.
* Added the public `NATIVE_API_AVAILABLE` constant for the few places that must
  behave differently without the native Nautilus library.
* Documented the linked API on docs.rs rather than the unlinked stubs.
* Made GLib criticals fail the unit test run, so a wrapper that hands GLib a
  value it should have rejected is caught even when the safe API still returns
  the right answer.
* Moved GObject property and Nautilus getter access behind two internal safe
  accessor traits. Accessors on a wrapper built from a null pointer now return
  a neutral value instead of reaching GLib.
* Required a `// SAFETY:` comment on every `unsafe` block outside test code,
  enforced by `clippy::undocumented_unsafe_blocks`.

## 0.8.0 (2022-07-27)

* Fixed Clippy lint errors and warnings.
* Bumped version due to trait function signature changes.

## 0.7.0 (2022-07-26)

* Update gtk-rs and related dependencies to version 0.15.
* Require GTK+ 3.20 and Rust 1.57.
* Match -sys crate's minor version to main crate's (0.7).

## 0.6.1 (2020-05-13)

* Make methods in the `MenuProvider` trait optional.

## 0.6.0 (2020-05-10)

* Add `get_background_items()`

## 0.5.0 (2020-04-29)

* Replace deprecated `ATOMIC_USIZE_INIT`.
* Update dependencies

## 0.4.0 (2019-04-18)

* Replace `static mut` with safer alternatives.
* Require Rust 1.27.

## 0.3.1 (2018-04-03)

* Minor performance improvement using `Cow` instead of `String` when possible.

## 0.3.0 (2018-02-19)

* Require GTK+ 3.18 and Rust 1.24.

## 0.2.3 (2018-02-19)

* Fix warnings on Rust 1.24.

## 0.2.1 (2016-11-20)

* Convenience functions: `Column::new()`, `PropertyPage::new()`, `FileInfo.add_attribute()`.
* `Menu` and `MenuItem` functions prefer borrowing rather than moving.

## 0.2.0 (2016-11-19)

* Added PropertyPageProvider.

## 0.1.2 (2016-11-16)

* Removed unnecessary unsafe code and argument mutability.

## 0.1.1 (2016-11-12)

* Fixed unnecessary `FileInfo` lifetime and argument mutability.

## 0.1.0 (2016-11-10)

* First public release.
* ColumnProvider, InfoProvider, and MenuProvider are implemented.
* Enough code to allow [tmsu-nautilus-rs](https://github.com/talklittle/tmsu-nautilus-rs) to work.
