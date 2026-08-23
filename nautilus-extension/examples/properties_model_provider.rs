#![warn(clippy::undocumented_unsafe_blocks)]

use nautilus_extension::gobject_ffi::GTypeModule;
use nautilus_extension::{
    nautilus_module, FileInfo, NautilusModule, PropertiesItem, PropertiesModel,
    PropertiesModelProvider,
};

struct DemoPropertiesModelProvider;

impl PropertiesModelProvider for DemoPropertiesModelProvider {
    fn get_models(&self, files: &[FileInfo]) -> Vec<PropertiesModel> {
        if files.len() != 1 {
            return Vec::new();
        }

        let file = &files[0];
        let name = file.name().unwrap_or_else(|| "unknown".to_string());
        let uri_scheme = file.uri_scheme().unwrap_or_else(|| "unknown".to_string());
        let mime_type = file.mime_type().unwrap_or_else(|| "unknown".to_string());

        vec![PropertiesModel::new(
            "Rust Demo",
            vec![
                PropertiesItem::new("Name length", name.chars().count().to_string()),
                PropertiesItem::new("URI scheme", uri_scheme),
                PropertiesItem::new("MIME type", mime_type),
            ],
        )]
    }
}

fn register(module: *mut GTypeModule) -> nautilus_extension::glib_ffi::GType {
    let mut module = NautilusModule::new(module, "RustDemoPropertiesModelProvider");
    module.add_properties_model_provider(DemoPropertiesModelProvider);
    module.register()
}

nautilus_module!(register);
