#![no_main]

use libfuzzer_sys::fuzz_target;
use nautilus_extension::{Column, Menu, MenuItem, PropertiesItem, PropertiesModel};

fn bounded_count(byte: Option<&u8>, max: usize) -> usize {
    byte.map(|value| (*value as usize) % (max + 1)).unwrap_or(0)
}

fn label(prefix: &str, value: u8) -> String {
    format!("{prefix}_{value}")
}

fuzz_target!(|data: &[u8]| {
    let column_count = bounded_count(data.first(), 8);
    let menu_count = bounded_count(data.get(1), 8);
    let property_count = bounded_count(data.get(2), 16);

    let mut columns = Vec::new();
    for index in 0..column_count {
        let value = data.get(3 + index).copied().unwrap_or(index as u8);
        columns.push(Column::new(
            label("ColumnName", value),
            label("attribute", value),
            label("Column Label", value),
            label("Column Description", value),
        ));
    }
    for column in columns {
        let _ = column.to_object();
    }

    let mut menu_items = Vec::new();
    for index in 0..menu_count {
        let value = data.get(12 + index).copied().unwrap_or(index as u8);
        let child = MenuItem::new(label("Child", value), label("Child Label", value));
        let submenu = Menu::new(vec![child]);
        menu_items.push(MenuItem::new(label("Item", value), label("Item Label", value)).with_submenu(submenu));
    }
    let menu = Menu::new(menu_items);
    let _ = menu.items();

    let mut properties = Vec::new();
    for index in 0..property_count {
        let value = data.get(24 + index).copied().unwrap_or(index as u8);
        properties.push(PropertiesItem::new(label("Name", value), label("Value", value)));
    }
    let model = PropertiesModel::new("Fuzzed Properties", properties);
    let _ = model.to_object();
});
