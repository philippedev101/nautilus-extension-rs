#!/usr/bin/env perl
use strict;
use warnings;
use Cwd qw(abs_path);
use File::Find;
use File::Spec;

my $require_gir = 0;
for my $arg (@ARGV) {
    if ($arg eq '--require-gir') {
        $require_gir = 1;
    } else {
        die "usage: $0 [--require-gir]\n";
    }
}

my $script = abs_path($0);
my ($volume, $directories) = File::Spec->splitpath($script);
my @parts = File::Spec->splitdir($directories);
pop @parts while @parts && $parts[-1] eq '';
pop @parts; # validation
pop @parts; # scripts
my $repo_root = File::Spec->catpath($volume, File::Spec->catdir(@parts), '');

my @expected_sys_symbols = qw(
    nautilus_operation_result_get_type
    nautilus_column_get_type
    nautilus_column_new
    nautilus_column_provider_get_type
    nautilus_column_provider_get_columns
    nautilus_file_info_get_type
    nautilus_file_info_create
    nautilus_file_info_create_for_uri
    nautilus_file_info_lookup
    nautilus_file_info_lookup_for_uri
    nautilus_file_info_list_copy
    nautilus_file_info_list_free
    nautilus_file_info_add_emblem
    nautilus_file_info_add_string_attribute
    nautilus_file_info_can_write
    nautilus_file_info_get_activation_uri
    nautilus_file_info_get_file_type
    nautilus_file_info_get_location
    nautilus_file_info_get_mime_type
    nautilus_file_info_get_mount
    nautilus_file_info_get_name
    nautilus_file_info_get_parent_info
    nautilus_file_info_get_parent_location
    nautilus_file_info_get_parent_uri
    nautilus_file_info_get_string_attribute
    nautilus_file_info_get_uri
    nautilus_file_info_get_uri_scheme
    nautilus_file_info_invalidate_extension_info
    nautilus_file_info_is_directory
    nautilus_file_info_is_gone
    nautilus_file_info_is_mime_type
    nautilus_info_provider_get_type
    nautilus_info_provider_update_file_info
    nautilus_info_provider_cancel_update
    nautilus_info_provider_update_complete_invoke
    nautilus_menu_get_type
    nautilus_menu_append_item
    nautilus_menu_get_items
    nautilus_menu_new
    nautilus_menu_item_get_type
    nautilus_menu_item_list_free
    nautilus_menu_item_new
    nautilus_menu_item_activate
    nautilus_menu_item_set_submenu
    nautilus_menu_provider_emit_items_updated_signal
    nautilus_menu_provider_get_type
    nautilus_menu_provider_get_file_items
    nautilus_menu_provider_get_background_items
    nautilus_properties_item_get_type
    nautilus_properties_item_new
    nautilus_properties_item_get_name
    nautilus_properties_item_get_value
    nautilus_properties_model_get_type
    nautilus_properties_model_new
    nautilus_properties_model_get_model
    nautilus_properties_model_get_title
    nautilus_properties_model_set_title
    nautilus_properties_model_provider_get_type
    nautilus_properties_model_provider_get_models
    NautilusColumnClass
    NautilusMenuClass
    NautilusMenuItemClass
    NautilusPropertiesItemClass
    NautilusPropertiesModelClass
    NautilusFileInfoInterface
    NautilusColumnProviderIface
    NautilusInfoProviderIface
    NautilusMenuProviderIface
    NautilusPropertiesModelProviderIface
    NautilusOperationHandle
    NautilusOperationResult
);

my @expected_safe_symbols = qw(
    NATIVE_API_AVAILABLE
    Column
    ColumnObject
    ColumnProvider
    ColumnProviderHandle
    ColumnSortOrder
    FileInfo
    FileInfoHandle
    FileInfoImpl
    FileInfoList
    InfoProvider
    InfoProviderHandle
    OperationHandle
    OperationResult
    OwnedGObject
    PendingUpdate
    UpdateCompleteCallback
    UpdateCompletion
    UpdateFileInfoOperation
    Menu
    MenuActivation
    MenuActivationTarget
    MenuItem
    MenuItemActivate
    MenuItemList
    MenuItemObject
    MenuItemType
    MenuObject
    MenuProvider
    MenuProviderHandle
    SignalHandlerId
    PropertiesItem
    PropertiesItemObject
    PropertiesModel
    PropertiesModelObject
    PropertiesModelProvider
    PropertiesModelProviderHandle
    NautilusModule
    NautilusModuleError
    IntoModuleTypes
    nautilus_module
    nautilus_module_initialize
    nautilus_module_list_types
    nautilus_module_shutdown
);

my @expected_safe_member_snippets = (
    'pub fn columns',
    'pub fn file_items',
    'pub fn background_items',
    'pub fn items',
    'pub fn models',
    'pub fn uri_or_empty',
    'pub fn uri_scheme_or_empty',
);

my %expected_gir = (
    class => { map { $_ => 1 } qw(Column Menu MenuItem PropertiesItem PropertiesModel) },
    interface => {
        map { $_ => 1 } qw(ColumnProvider FileInfo InfoProvider MenuProvider PropertiesModelProvider)
    },
    record => {
        map { $_ => 1 } qw(
            ColumnClass
            ColumnProviderInterface
            FileInfoInterface
            InfoProviderInterface
            MenuClass
            MenuItemClass
            MenuProviderInterface
            OperationHandle
            PropertiesItemClass
            PropertiesModelClass
            PropertiesModelProviderInterface
        )
    },
    enumeration => { map { $_ => 1 } qw(OperationResult) },
    function => {
        map { $_ => 1 } qw(
            create
            create_for_uri
            list_copy
            list_free
            lookup
            lookup_for_uri
            update_complete_invoke
            file_info_create
            file_info_create_for_uri
            file_info_list_copy
            file_info_list_free
            file_info_lookup
            file_info_lookup_for_uri
            info_provider_update_complete_invoke
            module_initialize
            module_list_types
            module_shutdown
        )
    },
);

sub read_file {
    my ($path) = @_;
    open my $fh, '<', $path or die "open $path: $!\n";
    local $/;
    return <$fh>;
}

sub read_safe_sources {
    my $src_dir = "$repo_root/nautilus-extension/src";
    my $text = '';
    find(
        sub {
            return unless /\.rs\z/;
            $text .= read_file($File::Find::name);
        },
        $src_dir
    );
    return $text;
}

sub discover_girs {
    return ($ENV{NAUTILUS_EXTENSION_RS_GIR}) if $ENV{NAUTILUS_EXTENSION_RS_GIR};

    my @candidates = (
        glob("$repo_root/Nautilus-4.*.gir"),
        glob('/usr/share/gir-1.0/Nautilus-4.*.gir'),
        glob('/usr/local/share/gir-1.0/Nautilus-4.*.gir'),
        glob('/nix/store/*nautilus*dev/share/gir-1.0/Nautilus-4.*.gir'),
        glob('/nix/store/*-nautilus-*/share/gir-1.0/Nautilus-4.*.gir'),
    );

    my %seen;
    return sort grep { -f $_ && !$seen{$_}++ } @candidates;
}

sub validate_symbols {
    my @errors;
    my $sys_text = read_file("$repo_root/nautilus-extension-sys/src/lib.rs");
    my $safe_text = read_safe_sources();

    for my $symbol (@expected_sys_symbols) {
        push @errors, "missing sys binding symbol: $symbol" if index($sys_text, $symbol) < 0;
    }

    for my $symbol (@expected_safe_symbols) {
        push @errors, "missing high-level API symbol: $symbol" if index($safe_text, $symbol) < 0;
    }

    for my $snippet (@expected_safe_member_snippets) {
        push @errors, "missing high-level API member snippet: $snippet"
            if index($safe_text, $snippet) < 0;
    }

    return @errors;
}

sub validate_tracking_doc {
    my @errors;
    my $path = "$repo_root/docs/API_TRACKING.md";

    if (!-f $path) {
        return ("missing API tracking document: docs/API_TRACKING.md");
    }

    my $text = read_file($path);

    for my $symbol (@expected_sys_symbols, @expected_safe_symbols) {
        push @errors, "API tracking document does not mention: $symbol"
            if index($text, $symbol) < 0;
    }

    for my $kind (sort keys %expected_gir) {
        for my $name (sort keys %{ $expected_gir{$kind} }) {
            push @errors, "API tracking document does not mention GIR $kind: $name"
                if index($text, $name) < 0;
        }
    }

    return @errors;
}

sub validate_gir {
    my ($path) = @_;
    my @errors;
    my $text = read_file($path);

    if ($text !~ /<namespace\s+name="Nautilus"\s+version="4\.[^"]*"/s) {
        push @errors, "$path does not expose Nautilus namespace version 4.x";
    }

    for my $kind (sort keys %expected_gir) {
        my %actual;
        while ($text =~ /<$kind\b[^>]*?\bname="([^"]+)"/g) {
            $actual{$1} = 1;
        }

        for my $name (sort keys %{ $expected_gir{$kind} }) {
            push @errors, "$path: GIR no longer exposes expected $kind: $name"
                unless $actual{$name};
        }

        for my $name (sort keys %actual) {
            push @errors, "$path: uncovered Nautilus GIR $kind: $name"
                unless $expected_gir{$kind}{$name};
        }
    }

    return @errors;
}

my @errors = validate_symbols();
push @errors, validate_tracking_doc();
my @girs = discover_girs();

if (!@girs) {
    my $message = 'Nautilus-4.x GIR metadata not found; static manifest checks were run';
    if ($require_gir) {
        push @errors, $message;
    } else {
        warn "warning: $message\n";
    }
} else {
    for my $gir (@girs) {
        push @errors, validate_gir($gir);
    }
}

if (@errors) {
    print STDERR "error: $_\n" for @errors;
    exit 1;
}

if (@girs) {
    print "api-surface: covered static manifest and GIR files:\n";
    print "  $_\n" for @girs;
} else {
    print "api-surface: covered static manifest\n";
}
