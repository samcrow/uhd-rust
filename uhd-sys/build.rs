extern crate bindgen;
extern crate metadeps;

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // This reads the metadata in Cargo.toml and sends Cargo the appropriate output to link the
    // libraries
    let libraries = metadeps::probe().unwrap();

    let uhd_include_path = libraries
        .get("uhd")
        .expect("uhd library not in map")
        .include_paths
        .get(0)
        .expect("no include path for UHD headers");
    generate_bindings(&uhd_include_path);
}

fn generate_bindings(include_path: &Path) {
    let usrp_header = include_path.join("uhd.h");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_path = out_dir.join("bindgen.rs");

    let mut builder = bindgen::builder()
        .whitelist_function("^uhd.+")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .header(usrp_header.to_string_lossy().to_owned())
        // Add the include directory to ensure that #includes in the header work correctly
        .clang_arg(format!("-I{}", include_path.to_string_lossy().to_owned()));

    // On Raspberry Pi devices, the include directories require some adjustment.
    let target = env::var("TARGET").expect("No TARGET environment variable");
    if target == "armv7-unknown-linux-gnueabihf" {
        builder = builder.clang_arg("-I/usr/lib/gcc/arm-linux-gnueabihf/8/include");
    }

    let bindings = builder.generate()
        .expect("Failed to generate bindings");
    bindings
        .write_to_file(out_path)
        .expect("Failed to write bindings to file");
}
