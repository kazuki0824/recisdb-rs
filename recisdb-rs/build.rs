extern crate cc;

fn main() {
    let mut cc = cc::Build::new();
    cc
        .file("src/IBonDriver.cpp")
        .cpp(true)
        .warnings(false);

    cc.compile("BonDriver_dynamic_cast_ffi");
}
