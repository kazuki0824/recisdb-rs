extern crate bindgen;
extern crate cc;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

//TODO: Refactor
fn main() {
    // println!("cargo:rerun-if-changed=src/inner_decoder/decoder.c");
    // println!("cargo:rerun-if-changed=src/inner_decoder/pipe_ecm.c");
    // println!("cargo:rerun-if-changed=src/inner_decoder/decoder.h");
    // println!("cargo:rerun-if-changed=src/inner_decoder/pipe_ecm.h");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let include_dir = format!("{}/{}", out_dir, "include");

    let mut cc = cc::Build::new();
    let pc = pkg_config::Config::new();

    //prepare ffi compile
    // cc.include(&include_dir)
    //     .flag_if_supported("-Wno-unused-parameter")
    //     .file("src/inner_decoder/pipe_ecm.c")
    //     .file("src/inner_decoder/decoder.c");

    //If libarib25 is found, then it'll continue. If not found, start build & deployment.
    if pc.target_supported() && !(cfg!(target_os = "windows")) {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        if let Ok(pcsc) = pc.probe("libpcsclite") {
            cc.includes(pcsc.include_paths.as_slice());
        }
        match pc.probe("libarib25") {
            Err(_e) => {
                //start self build
                let mut cm = cmake::Config::new("./externals/libaribb25");
                let res = cm.build();
                println!("cargo:rustc-link-search=native={}/lib", res.display());
                println!("cargo:rustc-link-lib=static=aribb25");
            }
            Ok(_b25) => {
                //cc.includes(b25.include_paths.as_slice());
            }
        }
    } else {
        //assume MSVC
        let mut cm = cmake::Config::new("./externals/libaribb25");
        cm.generator("Visual Studio 16").very_verbose(true);
        //MSVC + b25-rs(debug) + libarib25(debug) = fail
        //warning LNK4098: defaultlib \'MSVCRTD.../NODEFAULTLIB:library...
        cm.profile("Release");
        let res = cm.build();
        println!("cargo:rustc-link-search=native={}/lib", res.display());
        /* MSVC emits two different *.lib files, libarib25.lib and arib25.lib.
        The first one is a static library, but the other is an import library, which doesn't have any implemation. */
        println!("cargo:rustc-link-lib=static=libaribb25");
        println!("cargo:rustc-link-lib=dylib=winscard");
    }
}
