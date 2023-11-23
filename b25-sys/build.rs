extern crate pkg_config;

fn prep_cmake() -> cmake::Config {
    let mut cm = cmake::Config::new("./externals/libaribb25");
    cm.very_verbose(true);

    // Disable AVX2 for x64
    // NEON SIMD is also supported, but not all ARM SoCs support it, so build without it.
    cm.define("USE_AVX2", "OFF");

    if cfg!(windows) {
        if cfg!(target_env = "gnullvm") {
            unimplemented!("tier3 gnullvm")
        }
        match (cfg!(target_env = "gnu"), std::env::var("MSYSTEM")) {
            (false, _) => {
                cm.generator("Visual Studio 17 2022");

                println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_FEATURE");
                let features = std::env::var("CARGO_CFG_TARGET_FEATURE");
                let features = features.as_deref().unwrap_or_default();
                if features.contains("crt-static") {
                    // panic!();
                    cm.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded");
                }
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
    #[cfg(target_env = "gnu")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=static=aribb25");

    cm.profile("Release");
    cm
}

fn main() {
    // Check feat
    #[cfg(all(
        feature = "prioritized_card_reader",
        any(feature = "block00cbc", feature = "block40cbc")
    ))]
    compile_error!(
        "features `crate/prioritized_card_reader` and `crate/block**cbc` are mutually exclusive"
    );

    let mut pc = pkg_config::Config::new();
    pc.statik(false);
    if cfg!(windows) {
        let res = prep_cmake().build();
        println!("cargo:rustc-link-search=native={}/lib", res.display());
        println!("cargo:rustc-link-search=native={}/lib64", res.display());
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
