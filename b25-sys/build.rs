extern crate pkg_config;

fn prep_cmake() -> cmake::Config {
    let mut cm = cmake::Config::new("./externals/libaribb25");
    cm.very_verbose(true);

    // Enable AVX2 for x64
    // NEON SIMD is also supported, but not all ARM SoCs support it, so build without it.
    if cfg!(target_arch = "x86_64") {
        cm.define("USE_AVX2", "OFF");
    }

    if cfg!(windows) {
        if cfg!(target_env = "gnullvm") {
            unimplemented!("tier3 gnullvm")
        }
        match (cfg!(target_env = "gnu"), std::env::var("MSYSTEM")) {
            (false, _) => {
                cm.generator("Visual Studio 17 2022");

                #[cfg(target_feature = "crt-static")]
                cm.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded");
            }
            (true, Ok(sys_name)) if sys_name.to_lowercase().contains("mingw") => {
                cm.generator("Ninja");
            }
            (true, Ok(sys_name)) if sys_name.to_lowercase().contains("ucrt") => {
                cm.generator("Ninja");
            }
            (true, Ok(sys_name)) => {
                panic!("target_env:={sys_name} not supported.")
            }
            (true, _) => {
                cm.generator("MinGW Makefiles");
            }
        }
    }

    // Staticaly link against libaribb25.so or aribb25.lib.
    println!("cargo:rustc-link-lib=static=aribb25");

    cm.profile("Release");
    cm
}

fn main() {
    // Check feat
    #[cfg(all(
        feature = "prioritized_card_reader",
        feature = "block00cbc",
    ))]
    compile_error!(
        "features `crate/prioritized_card_reader` and `crate/block**cbc` are mutually exclusive"
    );

    let pc = pkg_config::Config::new();
    if cfg!(windows) {
        let res = prep_cmake().build();
        println!("cargo:rustc-link-search=native={}/lib", res.display());
        println!("cargo:rustc-link-lib=dylib=winscard");
    } else if cfg!(target_os = "linux") {
        if pc.probe("libpcsclite").is_err() {
            panic!()
        }
        if pc.probe("libaribb25").is_err() || cfg!(feature = "prioritized_card_reader") {
            let res = prep_cmake().build();
            println!("cargo:rustc-link-search=native={}/lib", res.display());
            println!("cargo:rustc-link-search=native={}/lib64", res.display());
        }
    }
}
