#![allow(unexpected_cfgs)]

#[cfg(not(nautilus_extension_rs_skip_link))]
mod native {
    use nautilus_extension::{
        ColumnObject, MenuItemObject, MenuObject, PropertiesItemObject, PropertiesModel,
    };

    #[test]
    fn object_wrappers_survive_repeated_native_create_clone_and_drop() {
        for index in 0..128 {
            let suffix = index.to_string();

            let column = ColumnObject::new(
                format!("RustValidation::Column{suffix}"),
                format!("rust_validation_attribute_{suffix}"),
                format!("Validation {suffix}"),
                "Validation column",
            )
            .expect("NautilusColumn should be constructible");
            let column_clone = column.clone();
            assert_eq!(
                column_clone.label().as_deref(),
                Some(format!("Validation {suffix}").as_str())
            );
            assert!(column_clone.set_label("Validation Updated"));
            assert_eq!(column.label().as_deref(), Some("Validation Updated"));

            let menu = MenuObject::new().expect("NautilusMenu should be constructible");
            let item = MenuItemObject::new(
                format!("RustValidation::MenuItem{suffix}"),
                format!("Menu Item {suffix}"),
            )
            .expect("NautilusMenuItem should be constructible");
            let submenu = MenuObject::new().expect("submenu should be constructible");
            item.set_submenu(&submenu);
            assert!(item.submenu().is_some());
            menu.append_item(&item);
            let items = menu.get_items();
            assert_eq!(items.len(), 1);
            assert_eq!(
                items[0].name().as_deref(),
                Some(format!("RustValidation::MenuItem{suffix}").as_str())
            );

            let properties_item =
                PropertiesItemObject::new(format!("Name {suffix}"), format!("Value {suffix}"))
                    .expect("NautilusPropertiesItem should be constructible");
            assert_eq!(
                properties_item.name().as_deref(),
                Some(format!("Name {suffix}").as_str())
            );
            assert_eq!(
                properties_item.value().as_deref(),
                Some(format!("Value {suffix}").as_str())
            );

            let properties_model = PropertiesModel::new(
                format!("Model {suffix}"),
                vec![nautilus_extension::PropertiesItem::new(
                    format!("Row {suffix}"),
                    format!("Value {suffix}"),
                )],
            )
            .to_object()
            .expect("NautilusPropertiesModel should be constructible");
            assert_eq!(
                properties_model.title().as_deref(),
                Some(format!("Model {suffix}").as_str())
            );
            assert_eq!(properties_model.items().len(), 1);
        }
    }
}
