#![warn(clippy::undocumented_unsafe_blocks)]

use nautilus_extension::gobject_ffi::GTypeModule;
use nautilus_extension::{nautilus_module, FileInfo, MenuItem, MenuProvider, NautilusModule};

struct DemoMenuProvider;

impl MenuProvider for DemoMenuProvider {
    fn get_file_items(&self, files: &[FileInfo]) -> Vec<MenuItem> {
        if files.is_empty() {
            return Vec::new();
        }

        vec![
            MenuItem::new("RustDemoMenu::print-selected-uris", "Print selected URIs").on_activate(
                |activation| {
                    for file in activation.files() {
                        if let Some(uri) = file.uri() {
                            eprintln!("nautilus-extension-rs selected URI: {uri}");
                        }
                    }
                },
            ),
        ]
    }

    fn get_background_items(&self, current_folder: &FileInfo) -> Vec<MenuItem> {
        let folder = current_folder.clone();

        vec![
            MenuItem::new("RustDemoMenu::print-current-folder", "Print current folder")
                .on_activate(move |_| {
                    if let Some(uri) = folder.uri() {
                        eprintln!("nautilus-extension-rs current folder: {uri}");
                    }
                }),
        ]
    }
}

fn register(module: *mut GTypeModule) -> nautilus_extension::glib_ffi::GType {
    let mut module = NautilusModule::new(module, "RustDemoMenuProvider");
    module.add_menu_provider(DemoMenuProvider);
    module.register()
}

nautilus_module!(register);
