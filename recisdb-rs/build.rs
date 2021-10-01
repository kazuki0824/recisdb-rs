extern crate cc;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let bg = bindgen::builder()
        .allowlist_type("IBonDriver[1-9]?")
        .allowlist_function("CreateBonDriver")
        .allowlist_function("interface_check_[2-3]?")
        .header("src/IBonDriver.hpp")
        .dynamic_library_name("BonDriver")
        .dynamic_link_require_all(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
        
    
    let bindings = bg
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("BonDriver_binding.rs"))
        .expect("Couldn't write bindings");

    let mut cc = cc::Build::new();
    cc
        .file("src/IBonDriver.cpp")
        .cpp(true)
        .warnings(false);

    cc.compile("BonDriver_dynamic_cast_ffi");
}
