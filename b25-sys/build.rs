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

    //prepare ffi compile
    cc.include(&include_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .file("src/inner_decoder/pipe_ecm.c")
        .file("src/inner_decoder/decoder.c");
    //prepare bindings generation
    let bg = bg
        .derive_copy(false)
        .clang_arg(format!("-I{}", include_dir))
        .header("src/inner_decoder/decoder.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    //If libarib25 is found, then it'll continue. If not found, start build & deployment.
    if pc.target_supported() && !(cfg!(target_os = "windows")) {
        if let Ok(pcsc) = pc.probe("libpcsclite") {
            cc.includes(pcsc.include_paths.as_slice());
        }
        match pc.probe("libarib25") {
            Err(_e) => {
                //start self build
                let mut cm = cmake::Config::new("./externals/libarib25");
                cm.build();
            }
            Ok(_b25) => {
                //cc.includes(b25.include_paths.as_slice());
            }
        }
    } else {
        //TODO:MSVC build
        //+BonDriver
        let mut cm = cmake::Config::new("./externals/libarib25");
        cm.generator("Visual Studio 16").very_verbose(true);
        let res = cm.build();
    }

    //start ffi compilation
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
