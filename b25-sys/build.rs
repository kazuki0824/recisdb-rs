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
    cm.configure_arg("-DUSE_AVX2=ON");

    if cfg!(target_os = "windows") {
        if cfg!(target_env = "msvc") {
            cm.generator("Visual Studio 17 2022");
            /*
            MSVC + libaribb25(debug) = fail
            warning LNK4098: defaultlib \'MSVCRTD.../NODEFAULTLIB:library...
             */
            cm.profile("Release");
        } else if cfg!(target_env = "gnu") {
            cm.generator("MinGW Makefiles");
            println!("cargo:rustc-link-lib=dylib=ucrt");
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        println!("cargo:rustc-link-search=native=C:\\Windows\\System32");
        println!("cargo:rustc-link-lib=dylib=winscard");
    }

    let res = cm.build();
    println!("cargo:rustc-link-search=native={}/lib", res.display());

    // Staticaly link against libaribb25.so or aribb25.lib.
    println!("cargo:rustc-link-lib=static=aribb25");
}
