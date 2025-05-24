extern crate pkg_config;

use std::env::var;

#[derive(Clone)]
struct TargetVar {
    arch: Option<String>,
    env: Option<String>,
    feat: Option<String>,
    m_system: Option<String>,
    os: Option<String>,
    win: bool,
}

impl Default for TargetVar {
    fn default() -> Self {
        Self {
            arch: var("CARGO_CFG_TARGET_ARCH").ok(),
            env: var("CARGO_CFG_TARGET_ENV").ok(),
            feat: var("CARGO_CFG_TARGET_FEATURE").ok(),
            m_system: var("MSYSTEM").ok(),
            os: var("CARGO_CFG_TARGET_OS").ok(),
            win: var("CARGO_CFG_WINDOWS").is_ok(),
        }
    }
}

fn prep_cmake(cx: TargetVar) -> cmake::Config {
    let mut cm = cmake::Config::new("./externals/libaribb25");
    cm.very_verbose(true);

    // CMake 4.0
    cm.define("CMAKE_POLICY_VERSION_MINIMUM", "3.5");

    // Disble AVX2 for x64
    if matches!(cx.arch, Some(ref arch) if arch == "x86_64") {
        cm.define("USE_AVX2", "OFF");
    }

    if cx.win {
        match (
            cx.env.clone().unwrap_or_default().contains("gnu"),
            cx.m_system,
        ) {
            (false, _) => {
                cm.generator("Visual Studio 17 2022");

                if cx.feat.clone().unwrap_or_default().contains("crt-static") {
                    cm.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded");
                }
                if cx.arch.clone().unwrap_or_default().contains("aarch64") {
                    cm.define("USE_NEON", "ON");
                }
            }
            (true, Some(sys_name)) if sys_name.to_lowercase().contains("mingw") => {
                cm.generator("Ninja");
            }
            (true, Some(sys_name)) if sys_name.to_lowercase().contains("ucrt") => {
                cm.generator("Ninja");
            }
            (true, Some(sys_name)) => {
                panic!("target_env:={sys_name} not supported.")
            }
            (true, _) => {
                cm.generator("MinGW Makefiles");
            }
        }
    }

    // Staticaly link against libaribb25.so or aribb25.lib.
    if cx.env.clone().take().unwrap_or_default().contains("gnu") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
    println!("cargo:rustc-link-lib=static=aribb25");

    #[cfg(not(debug_assertions))]
    cm.profile("Release");
    cm
}

fn main() {
    let cx = TargetVar::default();

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
    if cx.win {
        let res = prep_cmake(cx).build();
        println!("cargo:rustc-link-search=native={}/lib", res.display());
        println!("cargo:rustc-link-search=native={}/lib64", res.display());
        println!("cargo:rustc-link-lib=dylib=winscard");
    } else if cx.os.clone().unwrap_or_default().contains("linux") {
        if pc.probe("libpcsclite").is_err() {
            panic!("libpcsclite not found.")
        }
        if pc.probe("libaribb25").is_err() || cfg!(feature = "prioritized_card_reader") {
            let res = prep_cmake(cx).build();
            println!("cargo:rustc-link-search=native={}/lib", res.display());
            println!("cargo:rustc-link-search=native={}/lib64", res.display());
        }
    }
}
