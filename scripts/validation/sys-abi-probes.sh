#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

pkg=libnautilus-extension-4
if ! pkg-config --exists "$pkg" >/dev/null 2>&1; then
    pkg=libnautilus-extension
fi

if ! pkg-config --atleast-version=43 "$pkg" >/dev/null 2>&1; then
    echo "Nautilus API 4 development files were not found by pkg-config" >&2
    exit 1
fi

mkdir -p target/validation
probe_c=target/validation/sys-abi-probe.c
probe_bin=target/validation/sys-abi-probe

cat > "$probe_c" <<'C_EOF'
#include <glib-object.h>
#include <nautilus-extension.h>
#include <stddef.h>
#include <stdio.h>

#define STATIC_ASSERT(name, expr) typedef char static_assert_##name[(expr) ? 1 : -1]

static NautilusOperationResult probe_update_file_info(NautilusInfoProvider *provider,
                                                      NautilusFileInfo *file,
                                                      GClosure *closure,
                                                      NautilusOperationHandle **handle)
{
    (void) provider;
    (void) file;
    (void) closure;
    (void) handle;
    return NAUTILUS_OPERATION_COMPLETE;
}

static void probe_cancel_update(NautilusInfoProvider *provider,
                                NautilusOperationHandle *handle)
{
    (void) provider;
    (void) handle;
}

static GList *probe_columns(NautilusColumnProvider *provider)
{
    (void) provider;
    return NULL;
}

static GList *probe_menu_files(NautilusMenuProvider *provider, GList *files)
{
    (void) provider;
    (void) files;
    return NULL;
}

static GList *probe_menu_background(NautilusMenuProvider *provider,
                                    NautilusFileInfo *current_folder)
{
    (void) provider;
    (void) current_folder;
    return NULL;
}

static GList *probe_properties_models(NautilusPropertiesModelProvider *provider,
                                      GList *files)
{
    (void) provider;
    (void) files;
    return NULL;
}

int main(void)
{
    STATIC_ASSERT(operation_complete_value, NAUTILUS_OPERATION_COMPLETE == 0);
    STATIC_ASSERT(operation_failed_value, NAUTILUS_OPERATION_FAILED == 1);
    STATIC_ASSERT(operation_in_progress_value, NAUTILUS_OPERATION_IN_PROGRESS == 2);
    STATIC_ASSERT(column_class_prefix, offsetof(NautilusColumnClass, parent_class) == 0);
    STATIC_ASSERT(menu_class_prefix, offsetof(NautilusMenuClass, parent_class) == 0);
    STATIC_ASSERT(menu_item_class_activate_exists,
                  offsetof(NautilusMenuItemClass, activate) >= sizeof(GObjectClass));
    STATIC_ASSERT(properties_item_class_prefix,
                  offsetof(NautilusPropertiesItemClass, parent_class) == 0);
    STATIC_ASSERT(properties_model_class_prefix,
                  offsetof(NautilusPropertiesModelClass, parent_class) == 0);

    NautilusColumnProviderInterface column_provider = { 0 };
    NautilusInfoProviderInterface info_provider = { 0 };
    NautilusMenuProviderInterface menu_provider = { 0 };
    NautilusPropertiesModelProviderInterface properties_model_provider = { 0 };

    column_provider.get_columns = probe_columns;
    info_provider.update_file_info = probe_update_file_info;
    info_provider.cancel_update = probe_cancel_update;
    menu_provider.get_file_items = probe_menu_files;
    menu_provider.get_background_items = probe_menu_background;
    properties_model_provider.get_models = probe_properties_models;

    GType types[] = {
        nautilus_column_get_type(),
        nautilus_column_provider_get_type(),
        nautilus_file_info_get_type(),
        nautilus_info_provider_get_type(),
        nautilus_menu_get_type(),
        nautilus_menu_item_get_type(),
        nautilus_menu_provider_get_type(),
        nautilus_properties_item_get_type(),
        nautilus_properties_model_get_type(),
        nautilus_properties_model_provider_get_type(),
        nautilus_operation_result_get_type(),
    };

    for (gsize i = 0; i < G_N_ELEMENTS(types); i++) {
        if (types[i] == 0) {
            fprintf(stderr, "Nautilus GType probe %lu returned 0\n", (unsigned long) i);
            return 1;
        }
    }

    if (!g_type_is_a(nautilus_column_get_type(), G_TYPE_OBJECT) ||
        !g_type_is_a(nautilus_menu_get_type(), G_TYPE_OBJECT) ||
        !g_type_is_a(nautilus_menu_item_get_type(), G_TYPE_OBJECT) ||
        !g_type_is_a(nautilus_properties_item_get_type(), G_TYPE_OBJECT) ||
        !g_type_is_a(nautilus_properties_model_get_type(), G_TYPE_OBJECT)) {
        fprintf(stderr, "Nautilus object GType inheritance probe failed\n");
        return 1;
    }

    puts("sys-abi-probe: Nautilus C ABI probes passed");
    return 0;
}
C_EOF

cc "$probe_c" -o "$probe_bin" $(pkg-config --cflags --libs "$pkg" gobject-2.0 gio-2.0)
"$probe_bin"
