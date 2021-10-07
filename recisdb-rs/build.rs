extern crate bindgen;
extern crate cc;
extern crate glob;

use std::env;
use std::path::PathBuf;

fn main() {
    //TODO: detect current linker name
    if cfg!(target_os = "linux") {
        //println!("cargo:rustc-link-arg=-Wl,--unresolved-symbols=ignore-in-object-files");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-arg=/FORCE:UNRESOLVED");
    }
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    //IBon
    let bindings = {
        let bg = bindgen::builder()
            .allowlist_type("IBonDriver[1-9]?")
            .allowlist_function("CreateBonDriver")
            .header("src/IBonDriver.hpp")
            .dynamic_library_name("BonDriver")
            .dynamic_link_require_all(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks));

        bg
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings")
    };
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("BonDriver_binding.rs"))
        .expect("Couldn't write bindings");

    let mut compiler = cc::Build::new();

    let globs = &["src/IBonDriver.cpp", "src/vtable_resolver/*.cpp"];
    for pattern in globs {
        for path in glob::glob(pattern).unwrap() {
            let path = path.unwrap();
            compiler.file(path);
        }
    }

    compiler.cpp(true).warnings(false);

    compiler.compile("BonDriver_dynamic_cast_ffi");
}
