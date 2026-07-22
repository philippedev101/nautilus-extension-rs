fn main() {
    println!("cargo:rerun-if-env-changed=NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rustc-check-cfg=cfg(nautilus_extension_rs_skip_link)");

    if std::env::var_os("DOCS_RS").is_some()
        || std::env::var_os("NAUTILUS_EXTENSION_RS_SKIP_NAUTILUS4_PKG_CONFIG").is_some()
    {
        println!("cargo:rustc-cfg=nautilus_extension_rs_skip_link");
    }
}
