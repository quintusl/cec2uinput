#[cfg(target_os = "linux")]
fn main() {
    pkg_config::Config::new().probe("libcec").unwrap();
    build_cec_shim();
}

#[cfg(target_os = "macos")]
fn main() {
    println!("cargo:rustc-link-search=native=/usr/local/Cellar/libcec/7.1.1/lib");
    println!("cargo:rustc-link-lib=dylib=cec");
    println!("cargo:rustc-link-lib=dylib=p8-platform");
    build_cec_shim();
}

fn build_cec_shim() {
    println!("cargo:rerun-if-changed=src/cec_shim.cpp");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("src/cec_shim.cpp")
        .flag("-Wno-deprecated-copy-with-user-provided-copy");

    if cfg!(target_os = "macos") {
        build
            .include("/usr/local/Cellar/libcec/7.1.1/include")
            .include("/usr/local/Cellar/libcec/7.1.1/include/libcec");
    }

    build.compile("cec_shim");
}
