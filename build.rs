use std::path::{Path, PathBuf};
use std::{env, fs};

use bindgen::Builder;

use regex::Regex;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // uses prebuilt stub bindings
    } else {
        windows();
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // uses prebuilt stub bindings
    } else {
        linux();
    }
}

#[cfg(not(all(
    any(target_os = "windows", target_os = "linux"),
    target_arch = "x86_64"
)))]
fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // uses prebuilt stub bindings
    } else {
        panic!("This platform is not supported yet");
    }
}

#[allow(unused)]
fn windows() {
    let ndi_sdk_dir = PathBuf::from(env::var("NDI_SDK_DIR").expect("Failed to locate the NDI SDK"));

    println!(
        "cargo:rustc-link-search={}",
        ndi_sdk_dir.join("Lib/x64").display()
    );

    println!("cargo:rustc-link-lib=Processing.NDI.Lib.x64");

    let bindings = bindgen::Builder::default().header(
        ndi_sdk_dir
            .join("Include/Processing.NDI.Lib.h")
            .to_str()
            .unwrap(),
    );

    generate_bindings(bindings);

    fs::copy(
        ndi_sdk_dir.join("Bin/x64/Processing.NDI.Lib.x64.dll"),
        Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps/Processing.NDI.Lib.x64.dll"),
    )
    .unwrap();
}

#[allow(unused)]
fn linux() {
    let ndi_header_file =
        PathBuf::from(env::var("NDI_HEADER_DIR").unwrap_or("/usr/include".into()))
            .join("Processing.NDI.Lib.h");
    if !ndi_header_file.exists() {
        panic!(
            "You are missing the Processing.NDI.Lib.h header. Please install the Linux NDI SDK from https://ndi.video or through your package manager. If you have installed the SDK, but the header file is not located in /usr/include, set NDI_HEADER_DIR to point to the appropriate directory containing the header."
        );
    }

    println!("cargo::rustc-link-lib=dylib=ndi");
    println!("cargo::rerun-if-env-changed=NDI_HEADER_DIR");

    let bindings = bindgen::Builder::default().header(ndi_header_file.to_str().unwrap());

    generate_bindings(bindings);

    // fs::copy(
    //     ndi_sdk_dir.join("Bin/x64/Processing.NDI.Lib.x64.dll"),
    //     Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps/Processing.NDI.Lib.x64.dll"),
    // )
    // .unwrap();
}

fn generate_bindings(builder: Builder) {
    let mut bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_item("NDIlib_v6__bindgen_ty_[0-9]+")
        .blocklist_item("NDIlib_v5__bindgen_ty_[0-9]+");

    for v in ["v6", "v5", "v4_5", "v4", "v3", "v2"] {
        bindings = bindings
            .blocklist_item(format!("NDIlib_{}", v))
            .blocklist_function(format!("NDIlib_{}_load", v));
    }

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let default_bindings = bindings.to_string();

    // fs::write("./src/bindings/bindings.full.rs.bin", &default_bindings)
    //     .expect("Couldn't write docsrs bindings!");

    let stub_bindings = stub_bindings(default_bindings.clone());

    fs::write(out_path.join("bindings.rs"), default_bindings).expect("Couldn't write bindings!");

    fs::write("./src/bindings/bindings.docsrs.rs.bin", stub_bindings)
        .expect("Couldn't write docsrs bindings!");
}

/// Replaces `extern "C"` with stub functions for docs.rs builds
fn stub_bindings(mut bindings: String) -> String {
    bindings = bindings.replace("\r\n", "\n");

    let re = Regex::new(r#"(?s)(?:unsafe )?extern "C" \{([^}]+)\}"#).unwrap();

    let mut replacements = 0;

    bindings = re
        .replace_all(&bindings, |caps: &regex::Captures| {
            let fn_def = &caps[1];
            if fn_def.contains("pub fn") && !fn_def.contains("...") {
                replacements += 1;
                format!(
                    "{fn_def} {{\n    unimplemented!(\"Stub bindings\")\n}}",
                    fn_def = fn_def
                        .replace("pub fn", "pub unsafe fn")
                        .replace(";", "")
                        .replace("\n", "")
                        .trim()
                )
            } else {
                println!("Skipping function: {:?}", fn_def);
                "".to_string()
            }
        })
        .to_string();

    // fs::write("./src/bindings/bindings.test.rs.bin", &bindings)
    //     .expect("Couldn't write test bindings!");

    assert!(
        replacements > 0,
        "No unsafe extern \"C\" functions found in the bindings"
    );

    // The following assertion does not work with function pointer arguments, which are present in the Linux bindings.
    // Example snippet:
    // ```
    //  pub util_audio_to_interleaved_16s_v3: ::std::option::Option<
    //     unsafe extern "C" fn(
    //         p_src: *const NDIlib_audio_frame_v3_t,
    //         p_dst: *mut NDIlib_audio_frame_interleaved_16s_t,
    //     ) -> bool,
    // >,
    // ```

    // assert_eq!(
    //     bindings.matches("extern \"C\"").count(),
    //     0,
    //     "Found 'extern \"C\"' in the bindings, all should have been replaced"
    // );

    format!("// Stub build\n{bindings}")
}
