fn main() {
    println!("cargo:rerun-if-env-changed=NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rustc-check-cfg=cfg(nautilus_extension_rs_skip_link)");

    if std::env::var_os("DOCS_RS").is_some() {
        println!(
            "cargo:warning=Building documentation on docs.rs without probing the system Nautilus API 4 library"
        );
        println!("cargo:rustc-cfg=nautilus_extension_rs_skip_link");
        return;
    }

    let nautilus4 = probe_nautilus_extension();

    if let Err(errors) = nautilus4 {
        if std::env::var_os("NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG").is_some() {
            println!(
                "cargo:warning=Nautilus API 4 was not found by pkg-config; \
                 continuing without native Nautilus linking because \
                 NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG is set"
            );
            println!("cargo:rustc-cfg=nautilus_extension_rs_skip_link");
        } else {
            panic!(
                "Nautilus API 4 was not found by pkg-config. \
                 Install the Nautilus extension development package \
                 (for example libnautilus-extension-dev, nautilus-devel, \
                 or libnautilus-extension) and ensure PKG_CONFIG_PATH can find one \
                 of: libnautilus-extension-4.pc, libnautilus-extension.pc. \
                 pkg-config errors: {errors}"
            );
        }
    }
}

fn probe_nautilus_extension() -> Result<(), String> {
    let mut errors = Vec::new();

    for package in ["libnautilus-extension-4", "libnautilus-extension"] {
        match pkg_config::Config::new()
            .atleast_version("43")
            .probe(package)
        {
            Ok(_) => return Ok(()),
            Err(error) => errors.push(format!("{package}: {error}")),
        }
    }

    Err(errors.join("; "))
}
