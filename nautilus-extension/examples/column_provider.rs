#![warn(clippy::undocumented_unsafe_blocks)]

use nautilus_extension::gobject_ffi::GTypeModule;
use nautilus_extension::{
    nautilus_module, Column, ColumnProvider, FileInfo, InfoProvider, NautilusModule,
};

const DEMO_STATUS_ATTRIBUTE: &str = "demo_status";

struct DemoMetadataProvider;

impl ColumnProvider for DemoMetadataProvider {
    fn get_columns(&self) -> Vec<Column> {
        vec![Column::new(
            "RustDemo::status",
            DEMO_STATUS_ATTRIBUTE,
            "Demo Status",
            "Example metadata supplied by a Rust extension",
        )
        .visible(true)]
    }
}

impl InfoProvider for DemoMetadataProvider {
    fn update_file_info(&self, file: &mut FileInfo) {
        let status = match (file.uri_scheme(), file.mime_type()) {
            (Some(scheme), Some(mime_type)) => format!("{scheme}, {mime_type}"),
            (Some(scheme), None) => scheme,
            (None, Some(mime_type)) => mime_type,
            (None, None) => "unknown".to_string(),
        };

        file.add_string_attribute(DEMO_STATUS_ATTRIBUTE, &status);
    }
}

fn register(module: *mut GTypeModule) -> nautilus_extension::glib_ffi::GType {
    let mut module = NautilusModule::new(module, "RustDemoColumnProvider");
    module
        .add_column_provider(DemoMetadataProvider)
        .add_info_provider(DemoMetadataProvider);
    module.register()
}

nautilus_module!(register);
