use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    let ndi_sdk_dir = PathBuf::from(env::var("NDI_SDK_DIR").expect("Failed to locate the NDI SDK"));

    println!(
        "cargo:rustc-link-search={}",
        ndi_sdk_dir.join("Lib/x64").display()
    );

    println!("cargo:rustc-link-lib=Processing.NDI.Lib.x64");

    let bindings = bindgen::Builder::default()
        .header(
            ndi_sdk_dir
                .join("Include/Processing.NDI.Lib.h")
                .to_str()
                .unwrap(),
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    fs::copy(
        ndi_sdk_dir.join("Bin/x64/Processing.NDI.Lib.x64.dll"),
        Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps/Processing.NDI.Lib.x64.dll"),
    )
    .unwrap();
    // fs::copy(
    //     ndi_sdk_dir.join("Lib/x64/Processing.NDI.Lib.x64.lib"),
    //     Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps/Processing.NDI.Lib.x64.lib"),
    // )
    // .unwrap();
}
