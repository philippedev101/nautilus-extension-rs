#![no_main]

use libfuzzer_sys::fuzz_target;
use nautilus_extension::{
    Column, ColumnSortOrder, Menu, MenuItem, PropertiesItem, PropertiesModel,
};

fn field(fields: &[&str], index: usize) -> String {
    fields.get(index).copied().unwrap_or("").to_owned()
}

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let fields: Vec<&str> = text.split('\0').take(16).collect();

    let column = Column::new(
        field(&fields, 0),
        field(&fields, 1),
        field(&fields, 2),
        field(&fields, 3),
    )
    .visible(data.first().map(|byte| byte % 2 == 0).unwrap_or(false))
    .xalign(data.get(1).map(|byte| (*byte as f32) / 255.0).unwrap_or(0.0))
    .default_sort_order(if data.get(2).map(|byte| byte % 2 == 0).unwrap_or(false) {
        ColumnSortOrder::Ascending
    } else {
        ColumnSortOrder::Descending
    });
    let _ = column.to_object();

    let submenu = Menu::new(vec![MenuItem::new(field(&fields, 4), field(&fields, 5))]);
    let item = MenuItem::new(field(&fields, 6), field(&fields, 7))
        .sensitive(data.get(3).map(|byte| byte % 2 == 0).unwrap_or(true))
        .with_submenu(submenu);
    let _ = item.name();
    let _ = item.label();

    let properties = PropertiesModel::new(
        field(&fields, 8),
        vec![
            PropertiesItem::new(field(&fields, 9), field(&fields, 10)),
            PropertiesItem::new(field(&fields, 11), field(&fields, 12)),
        ],
    );
    let _ = properties.to_object();
});
