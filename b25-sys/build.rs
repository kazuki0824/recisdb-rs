extern crate pkg_config;

#[allow(unused_must_use)]
fn main() {
    // let out_dir = env::var("OUT_DIR").unwrap();
    // let out_path = PathBuf::from(&out_dir);
    // let include_dir = format!("{}/{}", out_dir, "include");

    let pc = pkg_config::Config::new();

    //If libaribb25 is found, then it'll continue. If not found, start build & deployment.
    pc.probe("libpcsclite");
    if pc.target_supported() && !(cfg!(target_os = "windows")) {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        if pc.probe("libarib25").is_err() {
            //start self build
            let mut cm = cmake::Config::new("./externals/libaribb25");
            let res = cm.build();
            println!("cargo:rustc-link-search=native={}/lib", res.display());
        }
    } else {
        //assume MSVC
        let mut cm = cmake::Config::new("./externals/libaribb25");
        cm.very_verbose(true);
        cm.configure_arg("-DUSE_AVX2=ON");
        //MSVC + b25-rs(debug) + libarib25(debug) = fail
        //warning LNK4098: defaultlib \'MSVCRTD.../NODEFAULTLIB:library...
        cm.profile("Release");
        let res = cm.build();
        println!("cargo:rustc-link-search=native={}/lib", res.display());
        /* MSVC emits two different *.lib files, libarib25.lib and arib25.lib.
        The first one is a static library, but the other is an import library, which doesn't have any implemation. */
        println!("cargo:rustc-link-lib=dylib=winscard");
    }
    println!("cargo:rustc-link-lib=static=aribb25");
}
