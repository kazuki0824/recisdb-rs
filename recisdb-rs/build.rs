extern crate cc;

fn main() {
    // let bg = bindgen::builder()
    //     .allowlist_type("IBonDriver[1-9]?")
    //     .allowlist_function("CreateBonDriver")
    //     .header("src/IBonDriver.hpp")
    //     .dynamic_library_name("BonDriver");

    let mut cc = cc::Build::new();
    cc
        .file("src/IBonDriver.cpp")
        .cpp(true)
        .warnings(false);

    cc.compile("BonDriver_dynamic_cast_ffi");
}
