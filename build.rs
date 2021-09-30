extern crate bindgen;
extern crate cc;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let include_dir = format!("{}/{}", out_dir, "include");

    let mut cc = cc::Build::new();
    let pc = pkg_config::Config::new();
    let bg = bindgen::Builder::default();

    cc.include(&include_dir)
        .flag("-Wno-unused-parameter")
        .file("src/inner_decoder/pipe_ecm.c")
        .file("src/inner_decoder/decoder.c");
    let bg = bg
        // The input header we would like to generate
        // bindings for.
        .derive_copy(false)
        .header("src/inner_decoder/decoder.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg(format!("-I{}", include_dir));

    if pc.target_supported() && !(cfg!(target_os = "windows")) {
        if let Ok(pcsc) = pc.probe("libpcsclite") {
            cc.includes(pcsc.include_paths.as_slice());
        }
        match pc.probe("libarib25") {
            Err(_e) => {
                //start self build
                let mut cm = cmake::Config::new("./libarib25");
                cm.build();
            }
            Ok(_b25) => {
                //cc.includes(b25.include_paths.as_slice());
            }
        }
    }
    cc.compile("b25_ffi");

    let bindings = bg
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("arib25_binding.rs"))
        .expect("Couldn't write bindings");
}
