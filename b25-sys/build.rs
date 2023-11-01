extern crate pkg_config;

#[allow(unused_must_use)]
#[allow(clippy::overly_complex_bool_expr)]
#[allow(clippy::nonminimal_bool)]
fn main() {
    // let out_dir = env::var("OUT_DIR").unwrap();
    // let out_path = PathBuf::from(&out_dir);
    // let include_dir = format!("{}/{}", out_dir, "include");

    let pc = pkg_config::Config::new();

    //If libaribb25 is found, then it'll continue. If not found, start build & deployment.
    pc.probe("libpcsclite");

    if pc.target_supported() && cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        if !pc.probe("libaribb25").is_err() {
            // Staticaly link against libaribb25.so or aribb25.lib.
            println!("cargo:rustc-link-lib=static=aribb25");
            return;
        }
        //start self build in Linux
    }

    let mut cm = cmake::Config::new("./externals/libaribb25");
    cm.very_verbose(true);
    // Enable AVX2 for x64
    // NEON SIMD is also supported, but not all ARM SoCs support it, so build without it.
    if cfg!(target_arch = "x86_64") {
        cm.configure_arg("-DUSE_AVX2=ON");
    }

    if cfg!(target_os = "windows") {
        if cfg!(target_env = "msvc") {
            cm.generator("Visual Studio 17 2022");
            /*
            MSVC + libaribb25(debug) = fail
            warning LNK4098: defaultlib \'MSVCRTD.../NODEFAULTLIB:library...
             */
            cm.profile("Release");
        } else if cfg!(target_env = "gnu") {
            match std::env::var("MSYSTEM") {
                Ok(sys_name) if sys_name.to_lowercase().contains("mingw64") => {
                    cm.generator("Ninja");
                    #[cfg(debug_assertions)]
                    println!("cargo:rustc-link-lib=msvcrtd");
                    #[cfg(not(debug_assertions))]
                    println!("cargo:rustc-link-lib=msvcrt");
                    println!("cargo:rustc-link-lib=ucrtbase");
                }
                Ok(sys_name) => {
                    panic!("target_env:={sys_name} not supported.")
                }
                _ => {
                    // TODO
                    cm.generator("MinGW Makefiles");
                    #[cfg(debug_assertions)]
                    println!("cargo:rustc-link-lib=msvcrtd");
                    #[cfg(not(debug_assertions))]
                    println!("cargo:rustc-link-lib=msvcrt");
                    println!("cargo:rustc-link-lib=ucrtbase");
                }
            }
        } else if cfg!(target_env = "gnullvm") {
            match std::env::var("MSYSTEM") {
                Ok(sys_name) if sys_name.to_lowercase().contains("ucrt") => {
                    cm.generator("Ninja");
                    println!("cargo:rustc-link-lib=ucrt");
                }
                Ok(sys_name) if sys_name.to_lowercase().contains("clang") => {
                    cm.generator("Ninja");
                    println!("cargo:rustc-link-lib=ucrt");
                }
                Ok(sys_name) => {
                    panic!("target_env:={sys_name} not supported.")
                }
                _ => {
                    // TODO
                    cm.generator("MinGW Makefiles");
                    println!("cargo:rustc-link-lib=ucrt");
                    // println!("cargo:rustc-link-lib=vcruntime140");
                }
            }
            // llvm-mingw
        }
        println!("cargo:rustc-link-search=native=C:\\Windows\\System32");
        println!("cargo:rustc-link-lib=dylib=winscard");
    }

    let res = cm.build();
    println!("cargo:rustc-link-search=native={}/lib", res.display());
    println!("cargo:rustc-link-search=native={}/lib64", res.display());

    // Staticaly link against libaribb25.so or aribb25.lib.
    println!("cargo:rustc-link-lib=static=aribb25");
}
