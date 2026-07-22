#include <gio/gio.h>
#include <glib.h>
#include <glib-object.h>
#include <gmodule.h>
#include <nautilus-extension.h>
#include <stdio.h>

typedef void (*ModuleInitializeFunc) (GTypeModule *module);
typedef void (*ModuleListTypesFunc) (const GType **types, int *num_types);
typedef void (*ModuleShutdownFunc) (void);

typedef struct
{
    GTypeModule parent_instance;
} ValidationTypeModule;

typedef struct
{
    GTypeModuleClass parent_class;
} ValidationTypeModuleClass;

G_DEFINE_TYPE (ValidationTypeModule, validation_type_module, G_TYPE_TYPE_MODULE)

typedef struct
{
    GObject parent_instance;
} ValidationFileInfo;

typedef struct
{
    GObjectClass parent_class;
} ValidationFileInfoClass;

static void validation_file_info_iface_init (NautilusFileInfoInterface *iface);

static GTypeModule *kept_type_modules[16];
static guint kept_type_module_count;

G_DEFINE_TYPE_WITH_CODE (ValidationFileInfo,
                         validation_file_info,
                         G_TYPE_OBJECT,
                         G_IMPLEMENT_INTERFACE (NAUTILUS_TYPE_FILE_INFO,
                                                validation_file_info_iface_init))

static gboolean
validation_type_module_load (GTypeModule *module)
{
    (void) module;
    return TRUE;
}

static void
validation_type_module_unload (GTypeModule *module)
{
    (void) module;
}

static void
validation_type_module_class_init (ValidationTypeModuleClass *klass)
{
    GTypeModuleClass *module_class = G_TYPE_MODULE_CLASS (klass);
    module_class->load = validation_type_module_load;
    module_class->unload = validation_type_module_unload;
}

static void
validation_type_module_init (ValidationTypeModule *module)
{
    (void) module;
}

static gboolean
validation_file_info_is_gone (NautilusFileInfo *file)
{
    (void) file;
    return FALSE;
}

static char *
validation_file_info_get_name (NautilusFileInfo *file)
{
    (void) file;
    return g_strdup ("nautilus-extension-rs-validation");
}

static char *
validation_file_info_get_uri (NautilusFileInfo *file)
{
    (void) file;
    return g_strdup ("file:///tmp/nautilus-extension-rs-validation");
}

static char *
validation_file_info_get_parent_uri (NautilusFileInfo *file)
{
    (void) file;
    return g_strdup ("file:///tmp");
}

static char *
validation_file_info_get_uri_scheme (NautilusFileInfo *file)
{
    (void) file;
    return g_strdup ("file");
}

static char *
validation_file_info_get_mime_type (NautilusFileInfo *file)
{
    (void) file;
    return g_strdup ("text/plain");
}

static gboolean
validation_file_info_is_mime_type (NautilusFileInfo *file, const char *mime_type)
{
    (void) file;
    return g_strcmp0 (mime_type, "text/plain") == 0;
}

static gboolean
validation_file_info_is_directory (NautilusFileInfo *file)
{
    (void) file;
    return FALSE;
}

static void
validation_file_info_add_emblem (NautilusFileInfo *file, const char *emblem_name)
{
    (void) file;
    (void) emblem_name;
}

static char *
validation_file_info_get_string_attribute (NautilusFileInfo *file, const char *attribute_name)
{
    (void) file;
    (void) attribute_name;
    return NULL;
}

static void
validation_file_info_add_string_attribute (NautilusFileInfo *file,
                                           const char *attribute_name,
                                           const char *value)
{
    (void) file;
    (void) attribute_name;
    (void) value;
}

static void
validation_file_info_invalidate_extension_info (NautilusFileInfo *file)
{
    (void) file;
}

static char *
validation_file_info_get_activation_uri (NautilusFileInfo *file)
{
    (void) file;
    return g_strdup ("file:///tmp/nautilus-extension-rs-validation");
}

static GFileType
validation_file_info_get_file_type (NautilusFileInfo *file)
{
    (void) file;
    return G_FILE_TYPE_REGULAR;
}

static GFile *
validation_file_info_get_location (NautilusFileInfo *file)
{
    (void) file;
    return g_file_new_for_uri ("file:///tmp/nautilus-extension-rs-validation");
}

static GFile *
validation_file_info_get_parent_location (NautilusFileInfo *file)
{
    (void) file;
    return g_file_new_for_uri ("file:///tmp");
}

static NautilusFileInfo *
validation_file_info_get_parent_info (NautilusFileInfo *file)
{
    (void) file;
    return NULL;
}

static GMount *
validation_file_info_get_mount (NautilusFileInfo *file)
{
    (void) file;
    return NULL;
}

static gboolean
validation_file_info_can_write (NautilusFileInfo *file)
{
    (void) file;
    return TRUE;
}

static void
validation_file_info_iface_init (NautilusFileInfoInterface *iface)
{
    iface->is_gone = validation_file_info_is_gone;
    iface->get_name = validation_file_info_get_name;
    iface->get_uri = validation_file_info_get_uri;
    iface->get_parent_uri = validation_file_info_get_parent_uri;
    iface->get_uri_scheme = validation_file_info_get_uri_scheme;
    iface->get_mime_type = validation_file_info_get_mime_type;
    iface->is_mime_type = validation_file_info_is_mime_type;
    iface->is_directory = validation_file_info_is_directory;
    iface->add_emblem = validation_file_info_add_emblem;
    iface->get_string_attribute = validation_file_info_get_string_attribute;
    iface->add_string_attribute = validation_file_info_add_string_attribute;
    iface->invalidate_extension_info = validation_file_info_invalidate_extension_info;
    iface->get_activation_uri = validation_file_info_get_activation_uri;
    iface->get_file_type = validation_file_info_get_file_type;
    iface->get_location = validation_file_info_get_location;
    iface->get_parent_location = validation_file_info_get_parent_location;
    iface->get_parent_info = validation_file_info_get_parent_info;
    iface->get_mount = validation_file_info_get_mount;
    iface->can_write = validation_file_info_can_write;
}

static void
validation_file_info_class_init (ValidationFileInfoClass *klass)
{
    (void) klass;
}

static void
validation_file_info_init (ValidationFileInfo *file)
{
    (void) file;
}

static void
update_complete_cb (NautilusInfoProvider *provider,
                    NautilusOperationHandle *handle,
                    NautilusOperationResult result,
                    gpointer user_data)
{
    (void) provider;
    (void) handle;
    (void) result;
    (void) user_data;
}

static NautilusFileInfo *
validation_file_info (void)
{
    return NAUTILUS_FILE_INFO (g_object_new (validation_file_info_get_type (), NULL));
}

static void
free_object_list (GList *objects)
{
    g_list_free_full (objects, g_object_unref);
}

static gboolean
probe_column_provider (GObject *object)
{
    GList *columns = nautilus_column_provider_get_columns (NAUTILUS_COLUMN_PROVIDER (object));
    for (GList *iter = columns; iter != NULL; iter = iter->next) {
        if (!G_IS_OBJECT (iter->data)) {
            g_printerr ("column provider returned a non-GObject entry\n");
            free_object_list (columns);
            return FALSE;
        }
    }

    free_object_list (columns);
    return TRUE;
}

static gboolean
probe_info_provider (GObject *object, NautilusFileInfo *file)
{
    GClosure *closure = g_cclosure_new (G_CALLBACK (update_complete_cb), NULL, NULL);
    NautilusOperationHandle *handle = NULL;

    NautilusOperationResult result =
        nautilus_info_provider_update_file_info (NAUTILUS_INFO_PROVIDER (object),
                                                 file,
                                                 closure,
                                                 &handle);

    if (result == NAUTILUS_OPERATION_IN_PROGRESS) {
        if (handle == NULL) {
            g_printerr ("info provider returned IN_PROGRESS without a handle\n");
            g_closure_unref (closure);
            return FALSE;
        }
        nautilus_info_provider_cancel_update (NAUTILUS_INFO_PROVIDER (object), handle);
    } else if (handle != NULL) {
        g_printerr ("info provider returned a handle for a synchronous result\n");
        g_closure_unref (closure);
        return FALSE;
    }

    g_closure_unref (closure);
    return TRUE;
}

static gboolean
probe_menu_provider (GObject *object, NautilusFileInfo *file)
{
    GList *files = g_list_append (NULL, file);
    GList *items = nautilus_menu_provider_get_file_items (NAUTILUS_MENU_PROVIDER (object), files);
    g_list_free (files);
    nautilus_menu_item_list_free (items);

    items = nautilus_menu_provider_get_background_items (NAUTILUS_MENU_PROVIDER (object), file);
    nautilus_menu_item_list_free (items);

    return TRUE;
}

static gboolean
probe_properties_model_provider (GObject *object, NautilusFileInfo *file)
{
    GList *files = g_list_append (NULL, file);
    GList *models =
        nautilus_properties_model_provider_get_models (NAUTILUS_PROPERTIES_MODEL_PROVIDER (object),
                                                       files);
    g_list_free (files);
    free_object_list (models);

    return TRUE;
}

static gboolean
probe_type (GType type, guint *provider_count)
{
    GObject *object = g_object_new (type, NULL);
    if (object == NULL) {
        g_printerr ("failed to instantiate registered GType %s\n", g_type_name (type));
        return FALSE;
    }

    NautilusFileInfo *file = validation_file_info ();
    if (file == NULL) {
        g_printerr ("failed to create NautilusFileInfo for provider probes\n");
        g_object_unref (object);
        return FALSE;
    }

    gboolean ok = TRUE;

    if (g_type_is_a (type, NAUTILUS_TYPE_COLUMN_PROVIDER)) {
        (*provider_count)++;
        ok = ok && probe_column_provider (object);
    }
    if (g_type_is_a (type, NAUTILUS_TYPE_INFO_PROVIDER)) {
        (*provider_count)++;
        ok = ok && probe_info_provider (object, file);
    }
    if (g_type_is_a (type, NAUTILUS_TYPE_MENU_PROVIDER)) {
        (*provider_count)++;
        ok = ok && probe_menu_provider (object, file);
    }
    if (g_type_is_a (type, NAUTILUS_TYPE_PROPERTIES_MODEL_PROVIDER)) {
        (*provider_count)++;
        ok = ok && probe_properties_model_provider (object, file);
    }

    g_object_unref (file);
    g_object_unref (object);
    return ok;
}

static void
keep_type_module_reachable (GTypeModule *module)
{
    if (kept_type_module_count < G_N_ELEMENTS (kept_type_modules)) {
        kept_type_modules[kept_type_module_count++] = module;
    }
}

int
main (int argc, char **argv)
{
    if (argc != 2) {
        g_printerr ("usage: %s EXTENSION_SO\n", argv[0]);
        return 2;
    }

    GModule *module = g_module_open (argv[1], G_MODULE_BIND_LOCAL);
    if (module == NULL) {
        g_printerr ("failed to open %s: %s\n", argv[1], g_module_error ());
        return 1;
    }

    ModuleInitializeFunc initialize = NULL;
    ModuleListTypesFunc list_types = NULL;
    ModuleShutdownFunc shutdown = NULL;

    if (!g_module_symbol (module, "nautilus_module_initialize", (gpointer *) &initialize) ||
        !g_module_symbol (module, "nautilus_module_list_types", (gpointer *) &list_types) ||
        !g_module_symbol (module, "nautilus_module_shutdown", (gpointer *) &shutdown)) {
        g_printerr ("%s is missing a Nautilus module entry point\n", argv[1]);
        g_module_close (module);
        return 1;
    }

    GTypeModule *type_module = g_object_new (validation_type_module_get_type (), NULL);
    if (type_module == NULL || !g_type_module_use (type_module)) {
        g_printerr ("failed to create/use validation GTypeModule\n");
        if (type_module != NULL) {
            g_object_unref (type_module);
        }
        g_module_close (module);
        return 1;
    }

    initialize (type_module);

    const GType *types = NULL;
    int num_types = 0;
    list_types (&types, &num_types);

    if (types == NULL || num_types <= 0) {
        g_printerr ("%s did not register any provider GTypes\n", argv[1]);
        shutdown ();
        g_type_module_unuse (type_module);
        g_module_close (module);
        return 1;
    }

    guint provider_count = 0;
    gboolean ok = TRUE;
    for (int i = 0; i < num_types; i++) {
        ok = ok && probe_type (types[i], &provider_count);
    }

    if (provider_count == 0) {
        g_printerr ("%s registered no Nautilus provider interfaces\n", argv[1]);
        ok = FALSE;
    }

    shutdown ();
    g_type_module_unuse (type_module);
    keep_type_module_reachable (type_module);
    g_module_close (module);

    if (!ok) {
        return 1;
    }

    g_print ("gmodule-harness: %s passed with %u provider interface(s)\n",
             argv[1],
             provider_count);
    return 0;
}
